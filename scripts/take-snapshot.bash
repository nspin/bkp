set -euET -o pipefail
shopt -s inherit_errexit

# TODO
# - Disable quoting of paths by find. See section "UNUSUAL FILENAMES" of man find(1).

subject="$1"
out="$2"

out_subject="$out/subject.txt"
out_sha256sum="$out/sha256sum.txt"
out_nodes="$out/nodes"
out_files="$out/files"
out_digests="$out/digests"

if [ ! -d "$subject" ]; then
    echo "error: '$subject' is not a directory" >&2
    exit 1
fi

if [ -e "$out" ]; then
    echo "error: '$out' already exists" >&2
    exit 1
fi

mkdir "$out"

(cd "$subject" && pwd) > "$out_subject"

find "$subject" -fprintf "$out_nodes" '%y %#m %s %P\0 %l\0\n' -a -type f -fprintf "$out_files" '%P\0'

(
    cd "$subject"
    while IFS= read -r -d $'\0' path; do
        sha256sum -bz "$path"
        echo
    done
) < "$out_files" > "$out_digests"

sha256sum -b "$out_nodes" "$out_digests" > "$out_sha256sum"
