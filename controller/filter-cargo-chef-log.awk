# The recipe uses placeholder targets, so these root-manifest warnings are expected.
/^warning: unused dependency `(argon2|async-trait|autographs-image-processing|axum|oracledb|reqwest|rsa|tracing-subscriber|x509-parser)`$/ {
    placeholder_warning = 1
    next
}

placeholder_warning && /^warning:/ {
    placeholder_warning = 0
}

placeholder_warning && /^$/ {
    placeholder_warning = 0
    next
}

placeholder_warning {
    next
}

/^warning: `autographs-controller` \(manifest\) generated [0-9]+ warnings?$/ {
    next
}

/Cargo.toml: edition is set on \[targets\] which is deprecated/ {
    next
}

{ print }
