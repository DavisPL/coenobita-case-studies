import subprocess
import argparse
import random
import time
import json

parser = argparse.ArgumentParser(
    prog='compare-crate-runtime',
    description='Runs interleaving performance experiments on the provided crates.'
)

parser.add_argument('a', help="path to crate a")
parser.add_argument('b', help="path to crate b")
parser.add_argument('-c', '--count', type=int, help="how many times the experiment should be run", default=30)
parser.add_argument('-w', '--warmup', type=int, help="number of warmup runs for each crate", default=5)
parser.add_argument('-v', '--verbose', action='store_true', help="show Cargo output", default=False)  
parser.add_argument('-o', '--output', help="path to save results JSON", default="results")

args = parser.parse_args()

results = {
    "a": [],
    "b": []
}

picks = ['a' for _ in range(args.count)] + ['b' for _ in range(args.count)]
random.shuffle(picks)

# Warm up
for crate in ['a', 'b']:
    path = (args.a if crate == 'a' else args.b) + "/Cargo.toml"
    
    for _ in range(args.warmup):
        subprocess.run(
            ["cargo", "test", "--release", "--manifest-path", path],
            stdout=subprocess.DEVNULL if not args.verbose else None,
            stderr=subprocess.DEVNULL if not args.verbose else None
        )

for pick in picks:
    path = (args.a if pick == 'a' else args.b) + "/Cargo.toml"

    start_time = time.perf_counter()
    subprocess.run(
        ["cargo", "test", "--release", "--manifest-path", path],
        stdout=subprocess.DEVNULL if not args.verbose else None,
        stderr=subprocess.DEVNULL if not args.verbose else None
    ) 
    end_time = time.perf_counter()
    
    execution_time = end_time - start_time
    results[pick].append(execution_time)

with open(args.output + ".json", 'w') as f:
    json.dump(results, f)

with open(args.output + ".txt", 'w') as f:
    f.write("=== a ===\n")

    for r in results["a"]:
        f.write(f"{r}\n")

    f.write("\n=== b ===\n")
    
    for r in results["b"]:
        f.write(f"{r}\n")