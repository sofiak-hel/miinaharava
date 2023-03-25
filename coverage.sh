cargo clean

llvm-cov report \
    $( \
      for file in \
        $( \
          RUSTFLAGS="-C instrument-coverage" \
            cargo test --tests --no-run --message-format=json \
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM - \
        ); \
      do \
        printf "%s %s " -object $file; \
      done \
    ) \
  --instr-profile=json5format.profdata --summary-only # and/or other options


# LLVM_PROFILE_FILE="../default_%m_%p.profraw" RUSTFLAGS="-C instrument-coverage" \
#     cargo test --tests

# llvm-profdata merge -sparse default_*.profraw -o json5format.profdata

# llvm-cov report \
#     --use-color --ignore-filename-regex='/.cargo/registry' \
#     --instr-profile=json5format.profdata \
#     --object target/debug/deps/lib-30768f9c53506dc5 \
#     --object target/debug/deps/json5format-fececd4653271682
# llvm-cov show \
#     --use-color --ignore-filename-regex='/.cargo/registry' \
#     --instr-profile=json5format.profdata \
#     --object target/debug/deps/lib-30768f9c53506dc5 \
#     --object target/debug/deps/json5format-fececd4653271682 \
#     --show-instantiations --show-line-counts-or-regions \
#     --Xdemangler=rustfilt | less -R
