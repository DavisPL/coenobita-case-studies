import subprocess
import time
import sys
import os

MODES = ['build', 'test']

def clean(crate, env):
	subprocess.run(
		["cargo", "clean", f"--manifest-path={crate}/Cargo.toml"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        env=env
	)

def build(crate, env):
	subprocess.run(
		["cargo", "build", f"--manifest-path={crate}/Cargo.toml", "--release"],
        # stdout=subprocess.DEVNULL,
        # stderr=subprocess.DEVNULL,
        env=env
	)

def test(crate, env):
	subprocess.run(
		["cargo", "test", f"--manifest-path={crate}/Cargo.toml", "--release"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        env=env
	)

def bench(crate, config):
	env = os.environ.copy()

	if config['wrapper'] is not None:
		env['RUSTC_WRAPPER'] = config['wrapper']

	if config['log'] is not None:
		env['COENOBITA_LOG_LEVEL'] = config['log']

	if config['optimize']:
		env['RUSTFLAGS'] = "-C opt-level=3"

	if config['mode'] == "build":
		# We are benchmarking build times... build to warm up
		clean(crate, env)
		for i in range(10):
			build(crate, env)
			clean(crate, env)

		start = time.perf_counter()
		for i in range(config['iters']):
			build(crate, env)
			clean(crate, env)

		end = time.perf_counter()
	else:
		# We are benchmarking test times... test to warm up
		clean(crate, env)
		for i in range(10):
			test(crate, env)

		start = time.perf_counter()
		for i in range(config['iters']):
			test(crate, env)

		end = time.perf_counter()

	return end - start

def configuration():
	args = sys.argv[1:]
	defaults = {
		'iters': 10,
		'wrapper': None,
		'log': None,
		'optimize': False
	}

	if args[0] in MODES:
		defaults['mode'] = args[0]
	else:
		print(f"First argument must be one of {MODES}")
		exit(1)

	defaults['crate'] = args[1]

	for first, second in zip(args[2:], args[3:]):
		if first == '-o' or second == '-o':
			defaults['optimize'] = True

		if first == '-i' or first == '--iters':
			defaults['iters'] = int(second)

		if first == '-w' or first == '--wrapper':
			defaults['wrapper'] = str(second)

		if first == '-l' or first == '--log':
			defaults['log'] = str(second)

	return defaults

def main():
    # First, grab the configuration
    config = configuration()

    # Then, benchmark the specified crate
    result = bench(config['crate'], config)
    print(result)

if __name__ == "__main__":
    main()
