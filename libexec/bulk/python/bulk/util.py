import os
import subprocess
from binascii import hexlify
from contextlib import contextmanager

def ensure_parent(path):
    path.parent.mkdir(parents=True, exist_ok=True)

@contextmanager
def proc_stdout(args):
    try:
        with subprocess.Popen(args, stdout=subprocess.PIPE) as proc:
            yield proc.stdout
    finally:
        if proc.returncode:
            raise subprocess.CalledProcessError(proc.returncode, args)

@contextmanager
def random_file_in(dir_path):
    while True:
        rand = hexlify(os.urandom(16)).decode('ascii')
        path = dir_path / rand
        if not path.exists():
            ensure_parent(path)
            path.touch()
            break
    try:
        yield path
    finally:
        path.unlink()
