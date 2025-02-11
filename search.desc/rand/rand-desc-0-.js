searchState.loadedDescShard("rand", 0, "Utilities for random number generation\nA marker trait used to indicate that an <code>RngCore</code> …\nThe type returned in the event of a RNG error.\nTypes which may be filled with random data\nUser-level interface for RNGs\nImplementation-level interface for RNGs\nSeed type, which is restricted to types …\nA random number generator that can be explicitly seeded.\nA marker trait used to indicate that a <code>TryRngCore</code> …\nA potentially fallible variant of <code>RngCore</code>\nGenerating random samples from probability distributions\nFill self with random data\nFill any type implementing <code>Fill</code> with random data\nFill any type implementing <code>Fill</code> with random data\nFill <code>dest</code> with random data.\nCreates a new instance of the RNG seeded via <code>getrandom</code>.\nCreate a new PRNG seeded from an infallible <code>Rng</code>.\nCreate a new PRNG using the given seed.\nAlias for <code>Rng::random</code>.\nAlias for <code>Rng::random</code>.\nAlias for <code>Rng::random_bool</code>.\nAlias for <code>Rng::random_bool</code>.\nAlias for <code>Rng::random_range</code>.\nAlias for <code>Rng::random_range</code>.\nAlias for <code>Rng::random_ratio</code>.\nAlias for <code>Rng::random_ratio</code>.\nReturn the next random <code>u32</code>.\nReturn the next random <code>u64</code>.\nConvenience re-export of common members\nReturn a random value via the <code>StandardUniform</code> distribution.\nReturn a random value via the <code>StandardUniform</code> distribution.\nReturn a bool with a probability <code>p</code> of being true.\nReturn a bool with a probability <code>p</code> of being true.\nReturn an iterator over <code>random</code> variates\nReturn an iterator over <code>random</code> variates\nGenerate a random value in the given range.\nGenerate a random value in the given range.\nReturn a bool with a probability of <code>numerator/denominator</code> …\nReturn a bool with a probability of <code>numerator/denominator</code> …\nRandom number generators and adapters\nSample a new value, using the given distribution.\nSample a new value, using the given distribution.\nCreate an iterator that generates values using the given …\nCreate an iterator that generates values using the given …\nCreate a new PRNG using a <code>u64</code> seed.\nSequence-related functionality\nFill <code>dest</code> entirely with random data.\nCreates a new instance of the RNG seeded via <code>getrandom</code> …\nCreate a new PRNG seeded from a potentially fallible <code>Rng</code>.\nReturn the next random <code>u32</code>.\nReturn the next random <code>u64</code>.\nWrap RNG with the <code>UnwrapErr</code> wrapper.\nSample a <code>u8</code>, uniformly distributed over ASCII letters and …\nThe Bernoulli distribution <code>Bernoulli(p)</code>.\nError type returned from <code>Bernoulli::new</code>.\nTypes (distributions) that can be used to create a random …\n<code>p &lt; 0</code> or <code>p &gt; 1</code>.\nAn iterator over a <code>Distribution</code>\nA <code>Distribution</code> which maps sampled values to type <code>S</code>\nA distribution to sample floating point numbers uniformly …\nA distribution to sample floating point numbers uniformly …\nThe Standard Uniform distribution\nSample values uniformly between two bounds.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nConstruct a new <code>Bernoulli</code> with the probability of success …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nMap sampled values to type <code>S</code>\nMap sampled values to type <code>S</code>\nConstruct a new <code>Bernoulli</code> with the given probability of …\nReturns the probability (<code>p</code>) of the distribution.\nGenerate a random value of <code>T</code>, using <code>rng</code> as the source of …\nCreate an iterator that generates random values of <code>T</code>, …\nCreate an iterator that generates random values of <code>T</code>, …\nDistributions over slices\nA distribution uniformly sampling numbers within a given …\nA distribution to uniformly sample elements of a slice\nError: empty slice\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCreate a new <code>Choose</code> instance which samples uniformly from …\nReturns the count of choices in this distribution\n<code>low &gt; high</code>, or equal in case of exclusive range.\nError type returned from <code>Uniform::new</code> and <code>new_inclusive</code>.\nInput or range <code>high - low</code> is non-finite. Not relevant to …\nHelper trait similar to <code>Borrow</code> but implemented only for …\nRange that supports generating a single sample efficiently.\nHelper trait for creating objects using the correct …\nThe <code>UniformSampler</code> implementation supporting type <code>X</code>.\nSample values uniformly between two bounds.\nThe back-end implementing <code>UniformSampler</code> for <code>char</code>.\nThe back-end implementing <code>UniformSampler</code> for <code>Duration</code>.\nThe back-end implementing <code>UniformSampler</code> for …\nThe back-end implementing <code>UniformSampler</code> for integer types.\nHelper trait handling actual uniform sampling.\nThe back-end implementing <code>UniformSampler</code> for <code>usize</code>.\nThe type sampled by this implementation.\nImmutably borrows from an owned value. See <code>Borrow::borrow</code>\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCheck whether the range is empty.\nConstruct self, with inclusive lower bound and exclusive …\nCreate a new <code>Uniform</code> instance, which samples uniformly …\nConstruct self, with inclusive bounds <code>[low, high]</code>.\nCreate a new <code>Uniform</code> instance, which samples uniformly …\nSample a value.\nSample from distribution, Lemire’s method, unbiased\nSample from distribution, Lemire’s method, unbiased\nSample from distribution, Lemire’s method, unbiased\nSample from distribution, Lemire’s method, unbiased\nSample from distribution, Lemire’s method, unbiased\nSample from distribution, Lemire’s method, unbiased\nSample from distribution, Lemire’s method, unbiased\nSample from distribution, Lemire’s method, unbiased\nSample from distribution, Lemire’s method, unbiased\nSample from distribution, Lemire’s method, unbiased\nGenerate a sample from the given range.\nSample a single value uniformly from a range with …\nSample a single value uniformly from a range with …\nSample single value, Canon’s method, biased\nSample single value, Canon’s method, biased\nSample single value, Canon’s method, biased\nSample single value, Canon’s method, biased\nSample single value, Canon’s method, biased\nSample single value, Canon’s method, biased\nSample single value, Canon’s method, biased\nSample single value, Canon’s method, biased\nSample single value, Canon’s method, biased\nSample single value, Canon’s method, biased\nAn interface over the operating-system’s random data …\nA wrapper around any PRNG that implements <code>BlockRngCore</code>, …\nA strong, fast (amortized), non-portable RNG\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nMock random number generator\nCreate a new <code>ReseedingRng</code> from an existing PRNG, combined …\nImmediately reseed the generator\nA mock generator yielding very predictable output\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCreate a <code>StepRng</code>, yielding an arithmetic sequence starting …\nExtension trait on indexable lists, providing random …\nExtension trait on indexable lists, providing random …\nExtension trait on iterators, providing random sampling …\nExtension trait on slices, providing shuffling methods.\nUniformly sample one element\nUniformly sample one element\nUniformly sample one element\nUniformly sample one element\nUniformly sample a fixed-size array of distinct elements …\nUniformly sample a fixed-size array of distinct elements …\nUniformly sample <code>amount</code> distinct elements into a buffer\nUniformly sample <code>amount</code> distinct elements into a buffer\nUniformly sample one element (mut)\nUniformly sample one element (mut)\nUniformly sample one element (stable)\nUniformly sample one element (stable)\nLow-level API for sampling indices\nTrue when the length is zero\nTrue when the length is zero\nThe length\nShuffle a slice in place, but exit early.\nShuffle a mutable slice in place.\nRandomly sample exactly <code>N</code> distinct indices from <code>0..len</code>, and")