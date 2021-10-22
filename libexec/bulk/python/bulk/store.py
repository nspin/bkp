import os
import re
import subprocess
from pathlib import Path

import logging
logger = logging.getLogger(__name__)

from bulk.hash import hash_file
from bulk.util import ensure_parent, random_file_in

__all__ = [
    'BlobStoreR',
    'BlobStoreRW',
    ]

digest_re = re.compile('(?P<a>[0-9a-h]{3})(?P<b>[0-9a-h]{61})')

class BlobStoreR:

    def __init__(self, path):
        self.path = Path(path)

    def blob_dir(self):
        return self.path / 'blobs'

    def partial_dir(self):
        return self.path / 'partial'

    def blob_relpath(self, digest):
        m = digest_re.fullmatch(digest)
        if m is None:
            raise Exception('invalid digest', digest)
        return Path(m['a'],  m['b'])

    def blob_path(self, digest):
        return self.blob_dir() / self.blob_relpath(digest)

    # (read-only)

    def have_blob(self, digest):
        return self.blob_path(digest).is_file()

    def check_blob(self, digest):
        if not self.have_blob(digest):
            return False
        observed_digest = hash_file(self.blob_path(digest))
        return digest == observed_digest

class BlobStoreRW(BlobStoreR):

    # (read-write)

    def store(self, src_path):
        digest = hash_file(src_path)
        if not self.have_blob(digest):
            blob_path = self.blob_path(digest)
            ensure_parent(blob_path)
            with random_file_in(self.partial_dir()) as tmp_path:
                subprocess.check_output(['cp', '--no-preserve=all', src_path, tmp_path])
                tmp_path.chmod(0o444) # -r--r--r--
                os.link(tmp_path, blob_path)
        return digest

    def clean(self):
        raise NotImplemented()
