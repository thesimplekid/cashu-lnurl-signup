run-dev:
    npx tailwindcss -i ./input.css -o ./public/css/out.css
    trunk serve
check:
    cargo fmt --check --all
    cargo clippy --all
test:
    cargo test

commit:
    cargo fmt --check
    cargo clippy
    trunk build --release
    git commit