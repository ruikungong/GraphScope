use pegasus::api::{Collect, CorrelatedSubTask, Iteration, Limit, Map, Sink, Merge};
use pegasus::JobConf;

// the most common case with early-stop
#[test]
fn limit_test_01() {
    let mut conf = JobConf::new("limit_test_01");
    conf.set_workers(2);
    let mut result = pegasus::run(conf, || {
        |input, output| {
            input
                .input_from(1..1000u32)?
                .flat_map(|i| Ok(0..i))?
                .repartition(|x: &u32| Ok(*x as u64))
                .flat_map(|i| Ok(0..i))?
                .limit(10)?
                .sink_into(output)
        }
    })
    .expect("build job failure");

    let mut count = 0;
    while let Some(Ok(d)) = result.next() {
        assert!(d < 1000);
        count += 1;
    }

    assert_eq!(count, 10);
}

#[test]
fn limit_test_with_tee() {
    let mut conf = JobConf::new("limit_test_with_tee");
    conf.set_workers(2);
    let mut result = pegasus::run(conf, || {
        |input, output| {
            let (left, right) = input.input_from(1..1000u32)?.flat_map(|i| Ok(0..i))?
                .repartition(|x: &u32| Ok(*x as u64))
                .flat_map(|i| Ok(0..i))?
                .copied()?;

            let left = left.limit(10)?;
            let right = right.limit(100)?;

            left.merge(right)?
                .sink_into(output)
        }
    })
        .expect("build job failure");


    let mut count = 0;
    while let Some(Ok(_)) = result.next() {
        count += 1;
    }

    assert_eq!(count, 110);
}

// early-stop with loop, triggered OUTSIDE loop
#[test]
fn limit_test_02() {
    let mut conf = JobConf::new("limit_test_02");
    conf.set_workers(2);
    conf.batch_capacity = 2;
    let mut result = pegasus::run(conf, || {
        |input, output| {
            input
                .input_from(1..1_000_000u32)?
                .iterate(2, |start| {
                    start
                        .repartition(|x: &u32| Ok(*x as u64))
                        .flat_map(|i| Ok(0..i))
                })?
                .limit(10)?
                .sink_into(output)
        }
    })
    .expect("build job failure");

    let mut count = 0;
    while let Some(Ok(d)) = result.next() {
        assert!(d < 1000);
        count += 1;
    }

    assert_eq!(count, 10);
}

// early-stop with loop, triggered INSIDE loop
#[test]
fn limit_test_03() {
    let mut conf = JobConf::new("limit_test_03");
    conf.set_workers(2);
    conf.batch_capacity = 2;
    let mut result = pegasus::run(conf, || {
        |input, output| {
            input
                .input_from(1..1000u32)?
                .iterate(2, |start| {
                    start
                        .repartition(|x: &u32| Ok(*x as u64))
                        .flat_map(|i| Ok(0..i * 1000))?
                        .limit(10)
                })?
                .sink_into(output)
        }
    })
    .expect("build job failure");

    let mut count = 0;
    while let Some(Ok(_)) = result.next() {
        count += 1;
    }

    assert_eq!(count, 10);
}

// early-stop with subtask, triggered OUTSIDE subtask
#[test]
fn limit_test_04() {
    let mut conf = JobConf::new("limit_test_04");
    conf.batch_capacity = 2;
    conf.scope_capacity = 10;
    conf.plan_print = true;
    conf.set_workers(2);
    let mut result = pegasus::run(conf, || {
        |input, output| {
            input
                .input_from(1..1000u32)?
                .apply(|sub| {
                    sub.flat_map(|i| Ok(0..i))?
                        .repartition(|x: &u32| Ok(*x as u64))
                        .flat_map(|i| Ok(0..i))?
                        .limit(1)? // mock has_any operator
                        .collect::<Vec<_>>()
                })?
                .limit(10)?
                .sink_into(output)
        }
    })
    .expect("build job failure");

    let mut count = 0;
    while let Some(Ok(d)) = result.next() {
        assert!(d.0 < 1000);
        assert!(d.1.len() <= 1);
        count += 1;
    }

    assert_eq!(count, 10);
}

// early-stop with subtask, triggered INSIDE subtask
#[test]
fn limit_test_05() {
    let mut conf = JobConf::new("limit_test_05");
    conf.set_workers(2);
    let mut result = pegasus::run(conf, || {
        |input, output| {
            input
                .input_from(1..500u32)?
                .apply(|sub| {
                    sub.flat_map(|i| Ok(0..i))?
                        .repartition(|x: &u32| Ok(*x as u64))
                        .flat_map(|i| Ok(0..i + 1))?
                        .limit(1)? // mock has_any operator
                        .collect::<Vec<_>>()
                })?
                .sink_into(output)
        }
    })
    .expect("build job failure");

    let mut count = 0;
    while let Some(Ok(d)) = result.next() {
        assert!(d.0 < 500);
        count += 1;
    }

    assert_eq!(count, 998);
}

// early-stop with subtask in loop, triggered INSIDE subtask
#[test]
fn limit_test_06() {
    let mut conf = JobConf::new("limit_test_06");
    conf.batch_capacity = 2;
    conf.set_workers(2);
    let mut result = pegasus::run(conf, || {
        |input, output| {
            input
                .input_from(1..11u32)?
                .iterate(2, |start| {
                    start
                        .flat_map(|i| Ok(i..i + 2))?
                        .apply(|sub| {
                            sub.repartition(|x: &u32| Ok(*x as u64))
                                .flat_map(|i| Ok(0..i * 1_000_000))?
                                .limit(1)? // mock has_any operator
                                .collect::<Vec<_>>()
                        })?
                        .map(|(i, _)| Ok(i))
                })?
                .sink_into(output)
        }
    })
    .expect("build job failure");

    let mut count = 0;
    while let Some(Ok(_)) = result.next() {
        count += 1;
    }

    assert_eq!(count, 80);
}

// early-stop with subtask in loop, triggered between OUTSIDE subtask but INSIDE loop
#[test]
#[ignore]// todo : wait fix;
fn limit_test_07() {
    let mut conf = JobConf::new("limit_test_07");
    conf.batch_capacity = 2;
    conf.plan_print = true;
    conf.set_workers(2);
    let mut result = pegasus::run(conf, || {
        |input, output| {
            input
                .input_from(1..100u32)?
                .iterate(2, |start| {
                    start
                        .flat_map(|i| Ok(0..i))?
                        .apply(|sub| {
                            sub.repartition(|x: &u32| Ok(*x as u64))
                                .flat_map(|i| Ok(0..i * 1_000_000))?
                                .limit(1)? // mock has_any operator
                                .collect::<Vec<_>>()
                        })?
                        .map(|(i, _)| Ok(i))?
                        .limit(10)
                })?
                .sink_into(output)
        }
    })
    .expect("build job failure");

    let mut count = 0;
    while let Some(Ok(_)) = result.next() {
        count += 1;
    }

    assert_eq!(count, 10);
}

// early-stop with subtask in loop, triggered OUTSIDE loop
#[test]
#[ignore] // todo wait fix;
fn limit_test_08() {
    let mut conf = JobConf::new("limit_test_08");
    conf.set_workers(2);
    let mut result = pegasus::run(conf, || {
        |input, output| {
            input
                .input_from(1..100u32)?
                .iterate(2, |start| {
                    start
                        .flat_map(|i| Ok(0..i))?
                        .apply(|sub| {
                            sub.repartition(|x: &u32| Ok(*x as u64))
                                .flat_map(|i| Ok(0..i))?
                                .limit(1)? // mock has_any operator
                                .collect::<Vec<_>>()
                        })?
                        .map(|(i, _)| Ok(i))
                })?
                .limit(10)?
                .sink_into(output)
        }
    })
    .expect("build job failure");

    let mut count = 0;
    while let Some(Ok(d)) = result.next() {
        assert!(d < 1000);
        count += 1;
    }

    assert_eq!(count, 10);
}
