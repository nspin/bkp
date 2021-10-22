set -euET -o pipefail
shopt -s inherit_errexit

subject="$1"
out="$2"

test ! -d "$out"

subject_file="$out/subject"
nodes_file="$out/nodes"
files_file="$out/files"
digests_file="$out/digests"

mkdir -p "$out"
printf "%s" "$subject" > "$subject_file"
find "$subject" -fprintf "$nodes_file" '%y %#m %P\0%l\0'
find "$subject" -type f -fprintf "$files_file" '%P\0'

(
    cd "$subject"
    while IFS= read -r -d $'\0' path; do
        sha256sum -bz "$path"
    done
) < "$files_file" > "$digests_file"
