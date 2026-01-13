_fzf_bash_completion_rust() {
    local output
    output=$(bash-fzf-tab-completion)
    local exit_code=$?

    if [ $exit_code -eq 0 ] && [ -n "$output" ]; then
        eval "$output"
    fi
}

bind -x '"\t": _fzf_bash_completion_rust'
