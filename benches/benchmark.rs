use core::num;
use std::{hint::black_box, time::Duration};
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkGroup, BenchmarkId};

use rand_core::{OsRng, RngCore};

use generic_array::typenum::{Sum, Diff, Quot, U, U1, U2};

use multiexp::multiexp_vartime;
use dalek_ff_group::{Scalar, EdwardsPoint};
use ciphersuite::{
  group::{ff::Field, Group, GroupEncoding},
  Ciphersuite, Ed25519, Selene, Helios,
};
use ec_divisors::{DivisorCurve, ScalarDecomposition};

use fcmps::{*, tree::hash_grow, Input, Output};

use generalized_bulletproofs_ec_gadgets::DiscreteLogParameters;

use monero_generators::{FCMP_U, FCMP_V, T};

use monero_fcmp_plus_plus::{*, sal::*};

fn random_output() -> Output<<Ed25519 as Ciphersuite>::G> {
  let O = <Ed25519 as Ciphersuite>::G::random(&mut OsRng);
  let I = <Ed25519 as Ciphersuite>::G::random(&mut OsRng);
  let C = <Ed25519 as Ciphersuite>::G::random(&mut OsRng);
  Output::new(O, I, C).unwrap()
}

fn random_path_including_output(
  layers: usize,
  output: &Output<<Ed25519 as Ciphersuite>::G>
) -> (Path<Curves>, TreeRoot<Selene, Helios>) {
  assert!(layers >= 1);

  let mut leaves = vec![];
  while leaves.len() < LAYER_ONE_LEN {
    leaves.push(random_output());
  }

  // set a random leaf to provided output
  let leaf_idx = usize::try_from(OsRng.next_u64() % u64::try_from(leaves.len()).unwrap()).unwrap();
  leaves[leaf_idx] = output.clone();

  let mut selene_hash = Some({
    let mut multiexp = vec![];
    for (scalar, point) in leaves
      .iter()
      .flat_map(|output| {
        [
          <Ed25519 as Ciphersuite>::G::to_xy(output.O()).unwrap().0,
          <Ed25519 as Ciphersuite>::G::to_xy(output.I()).unwrap().0,
          <Ed25519 as Ciphersuite>::G::to_xy(output.C()).unwrap().0,
        ]
      })
      .zip(SELENE_GENERATORS().g_bold_slice())
    {
      multiexp.push((scalar, *point));
    }
    SELENE_HASH_INIT() + multiexp_vartime(&multiexp)
  });
  let mut helios_hash = None;

  let mut curve_2_layers = vec![];
  let mut curve_1_layers = vec![];
  loop {
    if layers == 1 {
      break;
    }

    let mut curve_2_layer = vec![];
    while curve_2_layer.len() < LAYER_TWO_LEN {
      curve_2_layer.push(<Selene as Ciphersuite>::G::random(&mut OsRng));
    }
    let layer_len = curve_2_layer.len();
    curve_2_layer[usize::try_from(OsRng.next_u64()).unwrap() % layer_len] =
      selene_hash.take().unwrap();
    let curve_2_layer = curve_2_layer
      .into_iter()
      .map(|point| <Selene as Ciphersuite>::G::to_xy(point).unwrap().0)
      .collect::<Vec<_>>();

    helios_hash = Some({
      let mut multiexp = vec![];
      for (scalar, point) in curve_2_layer.iter().zip(HELIOS_GENERATORS().g_bold_slice()) {
        multiexp.push((*scalar, *point));
      }
      HELIOS_HASH_INIT() + multiexp_vartime(&multiexp)
    });

    curve_2_layers.push(curve_2_layer);

    if (1 + curve_1_layers.len() + curve_2_layers.len()) == layers {
      break;
    }

    let mut curve_1_layer = vec![];
    while curve_1_layer.len() < LAYER_ONE_LEN {
      curve_1_layer.push(<Helios as Ciphersuite>::G::random(&mut OsRng));
    }
    let layer_len = curve_1_layer.len();
    curve_1_layer[usize::try_from(OsRng.next_u64()).unwrap() % layer_len] =
      helios_hash.take().unwrap();
    let curve_1_layer = curve_1_layer
      .into_iter()
      .map(|point| <Helios as Ciphersuite>::G::to_xy(point).unwrap().0)
      .collect::<Vec<_>>();

    selene_hash = Some({
      let mut multiexp = vec![];
      for (scalar, point) in curve_1_layer.iter().zip(SELENE_GENERATORS().g_bold_slice()) {
        multiexp.push((*scalar, *point));
      }
      SELENE_HASH_INIT() + multiexp_vartime(&multiexp)
    });

    curve_1_layers.push(curve_1_layer);

    if (1 + curve_1_layers.len() + curve_2_layers.len()) == layers {
      break;
    }
  }

  let root = if let Some(selene_hash) = selene_hash {
    TreeRoot::<Selene, Helios>::C1(selene_hash)
  } else {
    TreeRoot::<Selene, Helios>::C2(helios_hash.unwrap())
  };

  (Path { output: output.clone(), leaves, curve_2_layers, curve_1_layers }, root)
}

fn random_paths_including_outputs(
  layers: usize,
  outputs: &[Output<<Ed25519 as Ciphersuite>::G>]
) -> (Vec<Path<Curves>>, TreeRoot<Selene, Helios>) {
  let paths = outputs.len();
  assert!(paths >= 1);
  assert!(paths <= LAYER_ONE_LEN.min(LAYER_TWO_LEN));

  let mut res = vec![];
  for output in outputs.iter() {
    let (path, _root) = random_path_including_output(layers, output);
    res.push(path);
  }

  // Pop each path's top layer
  // Then push a new top layer which is unified for all paths
  // 1st layer has a C1 root (so the top layer is the leaves)
  // 2nd layer has a C2 root (so the top layer is C1)
  // 3rd layer has a C1 root (so the top layer is C2)
  let root = if layers == 1 {
    let mut outputs = vec![];
    for path in &res {
      outputs.push(path.output);
    }
    while outputs.len() < LAYER_ONE_LEN {
      outputs.push(random_output());
    }
    let mut shuffled_outputs = vec![];
    while !outputs.is_empty() {
      let i = usize::try_from(OsRng.next_u64() % u64::try_from(outputs.len()).unwrap()).unwrap();
      shuffled_outputs.push(outputs.swap_remove(i));
    }

    for path in &mut res {
      path.leaves = shuffled_outputs.clone();
    }

    let mut new_leaves_layer = vec![];
    for output in shuffled_outputs {
      new_leaves_layer.push(<Ed25519 as Ciphersuite>::G::to_xy(output.O()).unwrap().0);
      new_leaves_layer.push(<Ed25519 as Ciphersuite>::G::to_xy(output.I()).unwrap().0);
      new_leaves_layer.push(<Ed25519 as Ciphersuite>::G::to_xy(output.C()).unwrap().0);
    }

    TreeRoot::C1(
      hash_grow(
        SELENE_GENERATORS(),
        SELENE_HASH_INIT(),
        0,
        <Selene as Ciphersuite>::F::ZERO,
        &new_leaves_layer,
      )
      .unwrap(),
    )
  } else if (layers % 2) == 0 {
    let mut branch = vec![];
    for path in &res {
      branch.push(
        <Selene as Ciphersuite>::G::to_xy(if let Some(branch) = path.curve_1_layers.last() {
          hash_grow(
            SELENE_GENERATORS(),
            SELENE_HASH_INIT(),
            0,
            <Selene as Ciphersuite>::F::ZERO,
            branch,
          )
          .unwrap()
        } else {
          let mut leaves_layer = vec![];
          for output in &path.leaves {
            leaves_layer.push(<Ed25519 as Ciphersuite>::G::to_xy(output.O()).unwrap().0);
            leaves_layer.push(<Ed25519 as Ciphersuite>::G::to_xy(output.I()).unwrap().0);
            leaves_layer.push(<Ed25519 as Ciphersuite>::G::to_xy(output.C()).unwrap().0);
          }

          hash_grow(
            SELENE_GENERATORS(),
            SELENE_HASH_INIT(),
            0,
            <Selene as Ciphersuite>::F::ZERO,
            &leaves_layer,
          )
          .unwrap()
        })
        .unwrap()
        .0,
      );
    }
    while branch.len() < LAYER_TWO_LEN {
      branch.push(<Helios as Ciphersuite>::F::random(&mut OsRng));
    }
    let mut shuffled_branch = vec![];
    while !branch.is_empty() {
      let i = usize::try_from(OsRng.next_u64() % u64::try_from(branch.len()).unwrap()).unwrap();
      shuffled_branch.push(branch.swap_remove(i));
    }

    for path in &mut res {
      *path.curve_2_layers.last_mut().unwrap() = shuffled_branch.clone();
    }

    TreeRoot::C2(
      hash_grow(
        HELIOS_GENERATORS(),
        HELIOS_HASH_INIT(),
        0,
        <Helios as Ciphersuite>::F::ZERO,
        &shuffled_branch,
      )
      .unwrap(),
    )
  } else {
    let mut branch = vec![];
    for path in &res {
      branch.push(
        <Helios as Ciphersuite>::G::to_xy({
          let branch = path.curve_2_layers.last().unwrap();
          hash_grow(
            HELIOS_GENERATORS(),
            HELIOS_HASH_INIT(),
            0,
            <Helios as Ciphersuite>::F::ZERO,
            branch,
          )
          .unwrap()
        })
        .unwrap()
        .0,
      );
    }
    while branch.len() < LAYER_ONE_LEN {
      branch.push(<Selene as Ciphersuite>::F::random(&mut OsRng));
    }
    let mut shuffled_branch = vec![];
    while !branch.is_empty() {
      let i = usize::try_from(OsRng.next_u64() % u64::try_from(branch.len()).unwrap()).unwrap();
      shuffled_branch.push(branch.swap_remove(i));
    }

    for path in &mut res {
      *path.curve_1_layers.last_mut().unwrap() = shuffled_branch.clone();
    }

    TreeRoot::C1(
      hash_grow(
        SELENE_GENERATORS(),
        SELENE_HASH_INIT(),
        0,
        <Selene as Ciphersuite>::F::ZERO,
        &shuffled_branch,
      )
      .unwrap(),
    )
  };

  // Verify each of these paths are valid
  for path in &res {
    assert!(path.leaves.iter().any(|output| output == &path.output));

    let mut leaves_layer = vec![];
    for output in &path.leaves {
      leaves_layer.push(<Ed25519 as Ciphersuite>::G::to_xy(output.O()).unwrap().0);
      leaves_layer.push(<Ed25519 as Ciphersuite>::G::to_xy(output.I()).unwrap().0);
      leaves_layer.push(<Ed25519 as Ciphersuite>::G::to_xy(output.C()).unwrap().0);
    }

    let mut c1_hash = Some(
      <Selene as Ciphersuite>::G::to_xy(
        hash_grow(
          SELENE_GENERATORS(),
          SELENE_HASH_INIT(),
          0,
          <Selene as Ciphersuite>::F::ZERO,
          &leaves_layer,
        )
        .unwrap(),
      )
      .unwrap()
      .0,
    );
    let mut c2_hash = None;

    let mut c1s = path.curve_1_layers.iter();
    let mut c2s = path.curve_2_layers.iter();
    loop {
      if let Some(layer) = c2s.next() {
        assert!(layer.iter().any(|leaf| leaf == &c1_hash.unwrap()));
        c1_hash = None;
        c2_hash = Some(
          <Helios as Ciphersuite>::G::to_xy(
            hash_grow(
              &HELIOS_GENERATORS(),
              HELIOS_HASH_INIT(),
              0,
              <Helios as Ciphersuite>::F::ZERO,
              layer,
            )
            .unwrap(),
          )
          .unwrap()
          .0,
        );
      } else {
        assert!(c1s.next().is_none());
      }

      if let Some(layer) = c1s.next() {
        assert!(layer.iter().any(|leaf| leaf == &c2_hash.unwrap()));
        c2_hash = None;
        c1_hash = Some(
          <Selene as Ciphersuite>::G::to_xy(
            hash_grow(
              SELENE_GENERATORS(),
              SELENE_HASH_INIT(),
              0,
              <Selene as Ciphersuite>::F::ZERO,
              layer,
            )
            .unwrap(),
          )
          .unwrap()
          .0,
        );
      } else {
        assert!(c2s.next().is_none());
        break;
      }
    }
    match root {
      TreeRoot::C1(root) => {
        assert_eq!(<Selene as Ciphersuite>::G::to_xy(root).unwrap().0, c1_hash.unwrap())
      }
      TreeRoot::C2(root) => {
        assert_eq!(<Helios as Ciphersuite>::G::to_xy(root).unwrap().0, c2_hash.unwrap())
      }
    }
  }

  (res, root)
}

fn random_output_blinds() -> OutputBlinds<<Ed25519 as Ciphersuite>::G> {
  let output_blinds_start = std::time::Instant::now();
  let res = OutputBlinds::new(
    OBlind::new(
    EdwardsPoint(T()),
      ScalarDecomposition::new(<Ed25519 as Ciphersuite>::F::random(&mut OsRng)).unwrap(),
    ),
    IBlind::new(
      EdwardsPoint(FCMP_U()),
      EdwardsPoint(FCMP_V()),
      ScalarDecomposition::new(<Ed25519 as Ciphersuite>::F::random(&mut OsRng)).unwrap(),
    ),
    IBlindBlind::new(
      EdwardsPoint(T()),
      ScalarDecomposition::new(<Ed25519 as Ciphersuite>::F::random(&mut OsRng)).unwrap(),
    ),
    CBlind::new(
      EdwardsPoint::generator(),
      ScalarDecomposition::new(<Ed25519 as Ciphersuite>::F::random(&mut OsRng)).unwrap(),
    ),
  );
  println!(
    "Output blinds took {}ms to calculate",
    (std::time::Instant::now() - output_blinds_start).as_millis()
  );
  res
}

fn blind_branches(
  branches: Branches<Curves>,
  output_blinds: &[OutputBlinds<<Ed25519 as Ciphersuite>::G>],
) -> BranchesWithBlinds<Curves> {
  let branch_blinds_start = std::time::Instant::now();
  let mut branches_1_blinds = vec![];
  for _ in 0 .. branches.necessary_c1_blinds() {
    branches_1_blinds.push(BranchBlind::<<Selene as Ciphersuite>::G>::new(
      SELENE_GENERATORS().h(),
      ScalarDecomposition::new(<Selene as Ciphersuite>::F::random(&mut OsRng)).unwrap(),
    ));
  }

  let mut branches_2_blinds = vec![];
  for _ in 0 .. branches.necessary_c2_blinds() {
    branches_2_blinds.push(BranchBlind::<<Helios as Ciphersuite>::G>::new(
      HELIOS_GENERATORS().h(),
      ScalarDecomposition::new(<Helios as Ciphersuite>::F::random(&mut OsRng)).unwrap(),
    ));
  }
  println!(
    "{} C1 branch blinds and {} C2 branch blinds took {}ms to calculate",
    branches.necessary_c1_blinds(),
    branches.necessary_c2_blinds(),
    (std::time::Instant::now() - branch_blinds_start).as_millis()
  );

  branches.blind(output_blinds.into_iter().cloned().collect(), branches_1_blinds, branches_2_blinds).unwrap()
}

fn verify_benchmark(c: &mut Criterion) {
  const MAX_NUM_PATHS: usize = 1;
  const TARGET_LAYERS: usize = 8;

  let signable_tx_hash = [0u8; 32];

  // all these vecs are length MAX_NUM_PATHS
  let mut O_x_openings = vec![];
  let mut O_y_openings = vec![];
  let mut O = vec![];
  let mut I = vec![];
  let mut C = vec![];
  let mut L = vec![];
  let mut output = vec![];
  let mut rerandomized_output = vec![];
  let mut output_blind = vec![];
  let mut input = vec![];
  let mut sal_proof = vec![];
  for _ in 0 .. MAX_NUM_PATHS {
    let x = Scalar::random(&mut OsRng);
    let y = Scalar::random(&mut OsRng);
    O_x_openings.push(x);
    O_y_openings.push(y);
    O.push(EdwardsPoint::generator() * x + EdwardsPoint(T()) * y);
    I.push(EdwardsPoint::random(&mut OsRng));
    C.push(EdwardsPoint::random(&mut OsRng));
    L.push(*I.last().unwrap() * x);
    output.push(Output::new(*O.last().unwrap(), *I.last().unwrap(), *C.last().unwrap()).unwrap());
    let rerando = RerandomizedOutput::new(&mut OsRng, *output.last().unwrap());
    rerandomized_output.push(rerando.clone());
    input.push(rerando.input());
    let opening = OpenedInputTuple::open(rerando.clone(), &x, &y).unwrap();
    let (L_, spend_auth_and_linkability) =
      SpendAuthAndLinkability::prove(&mut OsRng, signable_tx_hash, opening);
    assert_eq!(&L_, L.last().unwrap());
    sal_proof.push(spend_auth_and_linkability);
    output_blind.push(OutputBlinds::new(
      OBlind::new(
        EdwardsPoint(T()),
        ScalarDecomposition::new(rerando.o_blind()).unwrap(),
      ),
      IBlind::new(
        EdwardsPoint(FCMP_U()),
        EdwardsPoint(FCMP_V()),
        ScalarDecomposition::new(rerando.i_blind()).unwrap(),
      ),
      IBlindBlind::new(
        EdwardsPoint(T()),
        ScalarDecomposition::new(rerando.i_blind_blind()).unwrap(),
      ),
      CBlind::new(
        EdwardsPoint::generator(),
        ScalarDecomposition::new(rerando.c_blind()).unwrap(),
      ),
    ));
  }

  let mut test_cases = vec![];
  // j: number of tx inputs
  let mut j = 1;
  while j <= MAX_NUM_PATHS {
    let (paths, root) = random_paths_including_outputs(TARGET_LAYERS, &output[..j]);
    let branches = Branches::new(paths).unwrap();
    let blinded_branches = blind_branches(branches, &output_blind[..j]);
    let member_proof = Fcmp::prove(&mut OsRng, FCMP_PARAMS(), blinded_branches).unwrap();
    let fcmppp = FcmpPlusPlus::new(input.iter().cloned().zip(sal_proof.iter().cloned()).take(j).collect(), member_proof);
    test_cases.push((j, root, fcmppp));
    j *= 2;
  }

  // benchmark verification loop
  const MEASUREMENT_TIME: Duration = std::time::Duration::from_secs(30);
  let mut bench_group = c.benchmark_group("FCMP++ verify with N inputs");
  bench_group.measurement_time(MEASUREMENT_TIME);
  for (num_ins, root, fcmppp) in test_cases.iter() {
    //bench_group.throughput(Throughput::Bytes(*size as u64));
    bench_group.bench_with_input(BenchmarkId::from_parameter(num_ins), num_ins, |b, &size| {
          b.iter(|| {
            let mut ed_verifier = multiexp::BatchVerifier::new(1);
            let mut c1_verifier = generalized_bulletproofs::Generators::batch_verifier();
            let mut c2_verifier = generalized_bulletproofs::Generators::batch_verifier();

            fcmppp.verify(
              &mut OsRng, 
              &mut ed_verifier, 
              &mut c1_verifier, 
              &mut c2_verifier, 
              root.clone(), 
              TARGET_LAYERS, 
              signable_tx_hash,
              L.iter().take(*num_ins).cloned().collect()).unwrap();

            assert!(black_box(ed_verifier.verify_vartime()));
            assert!(black_box(SELENE_GENERATORS().verify(c1_verifier)));
            assert!(black_box(HELIOS_GENERATORS().verify(c2_verifier)));
          });
      });
  }
  bench_group.finish();
}

criterion_group!(benches, verify_benchmark);
criterion_main!(benches);
