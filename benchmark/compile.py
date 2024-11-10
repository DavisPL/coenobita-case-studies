import subprocess
import time
import sys
import os

EXCLUDE = ["originals", "benchmark", ".cargo", ".git"]

def clean(crate, env):
	subprocess.run(
		["cargo", "clean", f"--manifest-path={crate}/Cargo.toml"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        env=env
	)

def build(crate, env):
	start = time.time()

	subprocess.run(
		["cargo", "build", f"--manifest-path={crate}/Cargo.toml", "--release"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        env=env
	)

	end = time.time()
	return end - start

def bench(crate, config):
	times = []
	env = os.environ.copy()

	if config['wrapper'] is not None:
		env['RUSTC_WRAPPER'] = config['wrapper']

	for i in range(config['iters']):
		print(f"(Iteration {i})")
		times.append(build(crate, env))
		clean(crate, env)

	print(f"Compilation times for crate '{crate}'...")
	for time in times:
		print(f" > {time} seconds")

def configuration():
	args = sys.argv[1:]
	defaults = {
		'root': '.',
		'iters': 10,
		'wrapper': None
	}

	for first, second in zip(args, args[1:]):
		if (first == '-i' or first == '--iters'):
			defaults['iters'] = int(second)

		if (first == '-r' or first == '--root'):
			defaults['root'] = str(second)

		if (first == '-w' or first == '--wrapper'):
			defaults['wrapper'] = str(second)

	return defaults

def main():
    # First, grab the configuration
    config = configuration()

    # Then, benchmark every crate in the directory
    for directory in os.listdir(config['root']):
    	if os.path.isdir(directory) and directory not in EXCLUDE:
    		print(f"Checking '{directory}'...")
    		bench(directory, config)

if __name__ == "__main__":
    main()
