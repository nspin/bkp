import subprocess

__all__ = [
    'hash_file'
    ]

def hash_file(path):
    stdout = subprocess.check_output(['sha256sum', path])
    digest, _ = stdout.split(b'  ', 1)
    ensure_is_hex(64)(digest)
    return digest.decode('ascii')

hex_digits = frozenset(b'0123456789abcdef')

def ensure_is_hex(n):
    def f(s):
        ex = Exception(s, 'is not a hex string of', n, 'digits')
        if len(s) != n:
            raise ex
        for c in s:
            if c not in hex_digits:
                raise ex
    return f
