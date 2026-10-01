# Writes py-float-repr.txt.gz beside this file: python3 py_float_repr.py (CPython 3.11,
# float_repr_style 'short'). One repr(x) per line. The line is also the input: Python's repr
# round-trips, so parsing it gives back x exactly.
import gzip
import math
import os
import random
import struct
import sys
from decimal import Decimal

assert sys.float_info.mant_dig == 53 and sys.float_repr_style == "short"
rng = random.Random(0x6A1)


def from_bits(bits):
    return struct.unpack("<d", struct.pack("<Q", bits))[0]


def to_bits(x):
    return struct.unpack("<Q", struct.pack("<d", x))[0]


def sign(x):
    return -x if rng.random() < 0.5 else x


def exact_digits(x):
    return Decimal(x).as_tuple().digits


def repr_digits(x):
    return Decimal(repr(x)).normalize().as_tuple().digits


def is_tie(x):
    # repr kept D digits of an exact value with D + 1 digits ending in 5.
    exact = exact_digits(x)
    return exact[-1] == 5 and len(exact) == len(repr_digits(x)) + 1


out = []

# Ties: x = m * 2^-j with m odd has an exact decimal ending in 5. The shortest repr is a
# round-half-even tie when both neighbours one digit shorter round-trip.
ties = set()
while len(ties) < 8000:
    j = rng.randint(1, 64)
    m = rng.getrandbits(rng.randint(1, 53)) | 1
    x = math.ldexp(m, -j)
    if x != 0 and is_tie(x):
        ties.add(sign(x))
out += sorted(ties, key=abs)

# One ulp either side of a tie.
for x in sorted(ties, key=abs)[::4]:
    out += [math.nextafter(x, math.inf), math.nextafter(x, -math.inf)]

# Uniform bit patterns over all finite doubles.
out += [x for x in (from_bits(rng.getrandbits(64)) for _ in range(6000)) if math.isfinite(x)]

# Subnormals and the normal boundary.
out += [5e-324, -5e-324, 1e-323, from_bits(0x000FFFFFFFFFFFFF), sys.float_info.min, math.nextafter(sys.float_info.min, 1)]
out += [sign(from_bits(rng.getrandbits(52) >> rng.randint(0, 51))) for _ in range(1500)]

# Integers: small, log-uniform up to 2^53, around 2^53, exact powers of ten, and beyond.
out += [float(n) for n in range(0, 300)]
out += [sign(float(int(2 ** rng.uniform(0, 53)))) for _ in range(1500)]
out += [float(2**53 + d) for d in range(-8, 9)]
out += [float(10**k) for k in range(0, 24)]
out += [sign(float(int(10 ** rng.uniform(15, 300)))) for _ in range(500)]

# Every power of ten and power of two with its neighbours: the repr switches to exponent
# form below 1e-4 and at 1e16, and a power of two has an asymmetric rounding interval.
for k in range(-324, 309):
    x = float(f"1e{k}")
    out += [x, math.nextafter(x, math.inf), math.nextafter(x, -math.inf)]
for k in range(-1074, 1024):
    x = math.ldexp(1.0, k)
    out += [x, math.nextafter(x, -math.inf)]

# Short decimals like the pipeline's rounded coordinates and timings.
for _ in range(2000):
    places = rng.randint(0, 6)
    out.append(sign(round(rng.uniform(0, 10 ** rng.randint(0, 5)), places)))
out += [0.1 + 0.2, 1 / 3, 2 / 3, 1 / 60, 2.4, 0.8666666666666667, 123456789012345.6, 9007199254740993 / 8]

seen, lines = set(), []
for x in out:
    assert math.isfinite(x) and float(repr(x)) == x
    if to_bits(x) not in seen:
        seen.add(to_bits(x))
        lines.append(repr(x) + "\n")
path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "py-float-repr.txt.gz")
with gzip.GzipFile(path, "wb", compresslevel=9, mtime=0) as f:
    f.write("".join(lines).encode())
print(f"{len(lines)} values, {sum(is_tie(float(l)) for l in lines)} ties", file=sys.stderr)
