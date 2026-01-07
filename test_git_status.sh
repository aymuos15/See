#!/bin/bash
# Quick test to see if git2 can detect modifications

git status --short | head -5
echo "---"
git diff --name-only | head -5

# Try a simple rust snippet using git2 logic
cat > /tmp/test_status.sh << 'INNER'
#!/bin/bash
cd /home/localssk23/viewer
git2_test=$(cat <<'RUST'
use git2::Repository;
fn main() {
    if let Ok(repo) = Repository::discover(".") {
        if let Ok(statuses) = repo.statuses(None) {
            for entry in statuses.iter() {
                if let Some(path) = entry.path() {
                    let status = entry.status();
                    if status.is_wt_modified() || status.is_index_modified() {
                        println!("MODIFIED: {}", path);
                    }
                }
            }
        }
    }
}
RUST
)
echo "$git2_test"
INNER

chmod +x /tmp/test_status.sh
bash /tmp/test_status.sh
