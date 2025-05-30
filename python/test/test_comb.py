import opendp.prelude as dp
import pytest


def test_amplification():
    input_space = (
        dp.vector_domain(dp.atom_domain(bounds=(0.0, 10.0)), size=10),
        dp.symmetric_distance(),
    )
    meas = input_space >> dp.t.then_mean() >> dp.m.then_laplace(scale=0.5)

    amplified = dp.c.make_population_amplification(meas, population_size=100)
    print("amplified base laplace:", amplified([1.0] * 10))
    assert meas.check(2, 2.0 + 1e-6)
    assert not meas.check(2, 2.0)
    assert amplified.check(2, 1.494)
    assert not amplified.check(2, 0.494)


def test_fix_delta():
    input_space = dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float)

    m_gauss = dp.c.make_zCDP_to_approxDP(dp.m.make_gaussian(*input_space, 10.0))
    assert m_gauss.map(1.0).epsilon(1e-6) == 0.42994146883694934

    m_gauss_fixed = dp.c.make_fix_delta(m_gauss, 1e-6)
    assert m_gauss_fixed.map(1.0) == (0.42994146883694934, 1e-06)


def test_make_composition():
    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()
    composed = dp.c.make_composition(
        [
            input_space
            >> dp.t.then_count()
            >> dp.c.make_composition(
                [
                    dp.space_of(int) >> dp.m.then_laplace(scale=2.0),
                    dp.space_of(int) >> dp.m.then_laplace(scale=200.0),
                ]
            ),
            input_space
            >> dp.t.then_cast_default(bool)
            >> dp.t.then_cast_default(int)
            >> dp.t.then_count()
            >> dp.m.then_laplace(scale=2.0),
            input_space
            >> dp.t.then_cast_default(float)
            >> dp.t.then_impute_constant(0.0)
            >> dp.t.then_clamp((0.0, 10.0))
            >> dp.t.then_sum()
            >> dp.m.then_laplace(scale=2.0),
            dp.c.make_composition(
                [
                    input_space >> dp.t.then_count() >> dp.m.then_laplace(scale=2.0),
                    input_space
                    >> dp.t.then_count(TO=float)
                    >> dp.m.then_laplace(scale=2.0),
                    (
                        input_space
                        >> dp.t.then_cast_default(str)
                        >> dp.t.then_count_by_categories(categories=["0", "12", "22"])
                        >> dp.m.then_laplace(scale=2.0)
                    ),
                ]
            ),
        ]
    )

    print("Check:", composed.check(1, 2.0))
    print("Forward map:", composed.map(3))
    print("Invocation:", composed.invoke([22, 12]))


def test_deprecated_composition():
    input_space = dp.atom_domain(T=int), dp.absolute_distance(T=int)
    m_lap = dp.m.make_laplace(*input_space, 1.0)
    
    with pytest.deprecated_call():
        dp.c.make_basic_composition([m_lap] * 2)

    with pytest.deprecated_call():
        input_space >> dp.c.then_sequential_composition(
            m_lap.output_measure,
            1,
            [1.0] * 2,
        )


@pytest.mark.skip(reason="long-running process to detect potential memory leaks")
def test_make_composition_leak():

    # choose a vector-valued mechanism that should run quickly for large inputs
    # we want to add as little noise as possible, so that execution time is small
    space = dp.vector_domain(dp.atom_domain(T=int)), dp.l1_distance(T=int)
    meas = space >> dp.m.then_laplace(0.0)

    # memory usage remains the same when this line is commented,
    # supporting that AnyObject's free recursively frees children
    meas = dp.c.make_composition([meas])

    # watch for leaked AnyObjects with 1 million i32 values
    # memory would jump by ~40mb every iteration
    for i in range(1000):
        print("iteration", i)
        meas([0] * 1_000_000)


def test_make_composition_approx():
    input_space = dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float)
    composed_fixed = dp.c.make_composition(
        [
            dp.c.make_fix_delta(
                dp.c.make_zCDP_to_approxDP(dp.m.make_gaussian(*input_space, 1.0)), 1e-7
            )
        ]
        * 2
    )
    assert composed_fixed.map(1.0) == (11.341717670937543, 2e-07)


def test_cast_zcdp_approxdp():
    input_space = dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float)

    base_gaussian = input_space >> dp.m.then_gaussian(10.0, MO=dp.ZeroConcentratedDivergence)
    assert base_gaussian.map(1.0) == 0.005

    approx_gaussian = dp.c.make_zCDP_to_approxDP(base_gaussian)
    assert approx_gaussian.map(1.0).epsilon(1e-6) == 0.42994146883694934
    
def test_cast_azcdp_approxdp():
    m_azcdp = dp.m.make_user_measurement(
        dp.atom_domain(T=bool),
        dp.absolute_distance(T=float),
        dp.approximate(dp.zero_concentrated_divergence()),
        lambda x: x,
        # rho, and catastrophic delta
        lambda d_in: (d_in * 0.05, d_in * 1e-7),
    )
    m_asdp = dp.c.make_zCDP_to_approxDP(m_azcdp)
    curve, delta = m_asdp.map(1.0)
    assert delta == 1e-7

    m_adp = dp.c.make_fix_delta(m_asdp, delta=1e-6)
    assert m_adp.map(1.0) == (curve.epsilon(1e-6 - 1e-7), 1e-6)


def test_renyidp():
    import gc

    m_rdp = dp.m.make_user_measurement(
        dp.atom_domain(T=bool),
        dp.absolute_distance(T=float),
        dp.renyi_divergence(),
        lambda x: x,
        lambda d_in: (lambda alpha: d_in * alpha / 2.0),
    )
    rdp_curve = m_rdp.map(1.0)
    # this used to cause a use-after free,
    # as the ε(α) curve created by the map would not be kept alive
    gc.collect()
    assert rdp_curve(4.0) == 2.0


def test_make_approximate():
    input_space = dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float)

    # spot check that mechanisms get delta terms
    m_gauss = dp.c.make_approximate(dp.m.make_gaussian(*input_space, 1.0))
    assert m_gauss.map(1.0) == (0.5, 0.0)

    m_lap = dp.c.make_approximate(dp.m.make_laplace(*input_space, 1.0))
    assert m_lap.map(1.0) == (1.0, 0.0)

    # composition works properly
    m_comp = dp.c.make_composition(
        [
            m_lap,
            dp.c.make_fix_delta(
                dp.c.make_zCDP_to_approxDP(dp.m.make_gaussian(*input_space, 1.0)),
                delta=1e-6,
            ),
        ]
    )
    assert isinstance(m_comp.map(1.0), tuple)


def test_make_pureDP_to_zCDP():
    input_space = dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float)
    meas = dp.c.make_composition(
        [
            dp.c.make_pureDP_to_zCDP(dp.m.make_laplace(*input_space, 10.0)),
            dp.m.make_gaussian(*input_space, 10.0),
        ]
    )

    # (1/10)^2 / 2 + 1 / 10^2 / 2
    assert meas.map(1.0) == 0.010000000000000002


def test_make_fixed_approxDP_to_approxDP():
    input_space = dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float)
    fadp_meas = dp.c.make_approximate(dp.m.make_laplace(*input_space, 10.0))

    adp_meas = dp.c.make_fixed_approxDP_to_approxDP(fadp_meas)

    assert adp_meas.map(1.0).epsilon(delta=1e-7) == 0.1
