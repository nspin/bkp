import os
from argparse import ArgumentParser

import logging
logger = logging.getLogger(__name__)

GIT_DIR_ENV = 'GIT_DIR'
BLOB_STORE_ENV = 'BULK_BLOB_STORE'
LIBEXECDIR_ENV = 'BULK_LIBEXECDIR'

LOG_LEVELS = [logging.WARNING, logging.INFO, logging.DEBUG]

def parse_args():
    parser = ArgumentParser()
    parser.add_argument('--git-dir', metavar='GIT_DIR')
    parser.add_argument('--blob-store', metavar='BLOB_STORE')
    parser.add_argument('-v', '--verbose', dest='log_level', action='count', default=0)

    subparsers = parser.add_subparsers(dest='cmd')

    subparser = subparsers.add_parser('mount')
    subparser.add_argument('mountpoint', metavar='MOUNTPOINT')
    subparser.add_argument('tree', metavar='TREE', nargs='?', default='HEAD')

    subparser = subparsers.add_parser('snapshot')
    subparser.add_argument('subject', metavar='SUBJECT')
    subparser.add_argument('relative_path', metavar='RELATIVE_PATH')

    subparser = subparsers.add_parser('backup')
    subparser.add_argument('init', action='store_true')
    subparser.add_argument('target-git-dir', metavar='TARGET_GIT_DIR')

    #

    # subparser = subparsers.add_parser('take-snapshot')
    # subparser.add_argument('subject', metavar='SUBJECT')
    # subparser.add_argument('out', metavar='OUT')

    # subparser = subparsers.add_parser('plant-snapshot')
    # subparser.add_argument('snapshot', metavar='SNAPSHOT')

    # subparser = subparsers.add_parser('store-snapshot')
    # subparser.add_argument('tree', metavar='TREE')
    # subparser.add_argument('subject', metavar='SUBJECT')

    return parser.parse_args()

def update_args_from_env(args):
    args.git_dir = arg_or_env(args.git_dir, GIT_DIR_ENV)
    args.blob_store = arg_or_env(args.blob_store, BLOB_STORE_ENV)

def arg_or_env(arg, env_name):
    return arg if arg is not None else os.getenv(env_name)

def run(args):

    if args.cmd == 'mount':
        propagate_git_dir(args)
        propagate_blob_store(args)
        run_bash_script([args.mountpoint, args.tree], r'''
            tree="$(git rev-parse "$2"^{tree})"
            exec "$BULK_LIBEXECDIR/bulk-primitive" mount "$1" "$tree"
        ''')

    elif args.cmd == 'snapshot':
        propagate_git_dir(args)
        propagate_blob_store(args)
        run_bash_script([args.subject, args.relative_path], r'''
            subject="$1"
            relative_path="$2"
            tmp=./snapshot # TODO
            "$BULK_LIBEXECDIR/scripts/take-snapshot" "$subject" "$tmp"
            planted=$("$BULK_LIBEXECDIR/bulk-primitive" plant-snapshot "$tmp")
            tree=$(echo $planted | cut -d , -f 2)
            "$BULK_LIBEXECDIR/scripts/store-snapshot" $tree "$subject"
            # TODO rm tmp
            # TODO ensure parent
            git update-index --add --cacheinfo $planted,"$relative_path"
        ''')

    elif args.cmd == 'backup':
        propagate_git_dir(args)
        init = '1' if args.init else '0'
        run_bash_script([args.target_git_dir, init], r'''
            target_git_dir="$1"
            init=$2
            if [ $init = 1 ]; then
                # TODO
            fi
            # TODO
        ''')

    else:
        raise Exception('no command specified')

def run_bash_script(script_args, script):
    args = [
        'bash',
        '-euET', '-o', 'pipefail', '-O', 'inherit_errexit',
        '-c', script,
        '--',
        ]
    args.extend(script_args)
    os.execvp('bash', args)

def propagate_git_dir(args):
    if args.git_dir is None:
        raise Exception('missing --git-dir')
    os.environ[GIT_DIR_ENV] = args.git_dir

def propagate_blob_store(args):
    if args.blob_store is None:
        raise Exception('missing --blob-store')
    os.environ[BLOB_STORE_ENV] = args.blob_store

def apply_verbosity(args):
    level = LOG_LEVELS[min(args.log_level, len(LOG_LEVELS) - 1)]
    logging.basicConfig(level=level)

    # This doesn't include '__main__':
    # root_logger = logging.getLogger('bulk')
    # handler = logging.StreamHandler()
    # handler.setLevel(level)
    # handler.setFormatter(formatter)
    # root_logger.setLevel(level)
    # root_logger.addHandler(handler)

def main():
    args = parse_args()
    update_args_from_env(args)
    apply_verbosity(args)
    run(args)

if __name__ == '__main__':
    main()
