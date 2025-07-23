// src/shell.rs

pub const FISH_INIT_SCRIPT: &str = r#"
# Prevents this from being defined more than once
if not functions -q x
    function x --wraps cd --description "A smarter cd command powered by xneo"
        # Case 1: No arguments. Go home.
        if test -z "$argv"
            cd (command xneo)
            return
        end

        # Case 2: Check if it's a bookmark
        if test (count $argv) -eq 1
            set -l bookmark_path (command xneo bookmark get "$argv[1]" 2>/dev/null)
            if test -n "$bookmark_path"
                cd "$bookmark_path"
                return
            end
        end

        # Case 3: Argument is a direct, valid path. HIGHEST priority.
        if test -d "$argv[1]"
            cd "$argv[1]"
            return
        end
        
        # Case 4: Context-Aware Ancestor Matching
        set -l ancestor_match ""
        if test (count $argv) -eq 1
            set -l current_dir $PWD
            while test "$current_dir" != "/" -a "$current_dir" != "."
                if test (basename "$current_dir") = "$argv[1]"
                    set ancestor_match "$current_dir"
                    break
                end
                set current_dir (dirname "$current_dir")
            end
        end

        # If an ancestor match was found, jump there
        if test -n "$ancestor_match"
            cd "$ancestor_match"
        else
            # Case 5: Global Database Query
            set -l results (command xneo query $argv | string split -n '\n')
            set -l count (count $results)
            
            if test $count -eq 0
                echo "x: No match found for: $argv" >&2
                # Show similar suggestions
                set -l suggestions (command xneo query --suggest $argv 2>/dev/null | string split -n '\n')
                if test (count $suggestions) -gt 0
                    echo "Did you mean:" >&2
                    for suggestion in $suggestions[1..3]
                        echo "  $suggestion" >&2
                    end
                end
                return 1
            else if test $count -eq 1
                cd "$results[1]"
            else
                set -l choice (printf "%s\n" $results | fzf --height=40% --reverse --border --prompt="Select directory: ")
                if test -n "$choice"
                    cd "$choice"
                else
                    return 1
                end
            end
        end
    end
end

# History recording hook
if not functions -q __xneo_add_hook
    function __xneo_add_hook --on-variable PWD
        command xneo add "$PWD" &
    end
end

# Bookmark alias for convenience
if not functions -q xb
    function xb --description "Bookmark management for xneo"
        command xneo bookmark $argv
    end
end
"#;

pub const BASH_INIT_SCRIPT: &str = r#"
# xneo initialization for Bash

x() {
    # Case 1: No arguments, go home
    if [[ $# -eq 0 ]]; then
        cd "$(command xneo)"
        return
    fi

    # Case 2: Check if it's a bookmark
    if [[ $# -eq 1 ]]; then
        local bookmark_path
        bookmark_path=$(command xneo bookmark get "$1" 2>/dev/null)
        if [[ -n "$bookmark_path" ]]; then
            cd "$bookmark_path"
            return
        fi
    fi

    # Case 3: Direct path exists
    if [[ -d "$1" ]]; then
        cd "$1"
        return
    fi

    # Case 4: Context-aware ancestor matching
    if [[ $# -eq 1 ]]; then
        local current_dir="$PWD"
        local ancestor_match=""
        
        while [[ "$current_dir" != "/" && "$current_dir" != "." ]]; do
            if [[ "$(basename "$current_dir")" == "$1" ]]; then
                ancestor_match="$current_dir"
                break
            fi
            current_dir="$(dirname "$current_dir")"
        done
        
        if [[ -n "$ancestor_match" ]]; then
            cd "$ancestor_match"
            return
        fi
    fi

    # Case 5: Database query
    local results
    mapfile -t results < <(command xneo query "$@")
    
    case ${#results[@]} in
        0)
            echo "x: No match found for: $*" >&2
            # Show suggestions
            local suggestions
            mapfile -t suggestions < <(command xneo query --suggest "$@" 2>/dev/null)
            if [[ ${#suggestions[@]} -gt 0 ]]; then
                echo "Did you mean:" >&2
                printf "  %s\n" "${suggestions[@]:0:3}" >&2
            fi
            return 1
            ;;
        1)
            cd "${results[0]}"
            ;;
        *)
            local choice
            choice=$(printf "%s\n" "${results[@]}" | fzf --height=40% --reverse --border --prompt="Select directory: ")
            if [[ -n "$choice" ]]; then
                cd "$choice"
            else
                return 1
            fi
            ;;
    esac
}

# History recording
__xneo_add_hook() {
    command xneo add "$PWD" &
}

# Set up PROMPT_COMMAND
if [[ "$PROMPT_COMMAND" != *"__xneo_add_hook"* ]]; then
    PROMPT_COMMAND="${PROMPT_COMMAND:+$PROMPT_COMMAND; }__xneo_add_hook"
fi

# Bookmark alias
xb() {
    command xneo bookmark "$@"
}

# Completion for x command
_x_completion() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local suggestions
    mapfile -t suggestions < <(command xneo query --suggest "$cur" 2>/dev/null)
    COMPREPLY=($(compgen -W "${suggestions[*]}" -- "$cur"))
}

complete -F _x_completion x
"#;

pub const ZSH_INIT_SCRIPT: &str = r#"
# xneo initialization for Zsh

x() {
    # Case 1: No arguments, go home
    if [[ $# -eq 0 ]]; then
        cd "$(command xneo)"
        return
    fi

    # Case 2: Check if it's a bookmark
    if [[ $# -eq 1 ]]; then
        local bookmark_path
        bookmark_path=$(command xneo bookmark get "$1" 2>/dev/null)
        if [[ -n "$bookmark_path" ]]; then
            cd "$bookmark_path"
            return
        fi
    fi

    # Case 3: Direct path exists
    if [[ -d "$1" ]]; then
        cd "$1"
        return
    fi

    # Case 4: Context-aware ancestor matching
    if [[ $# -eq 1 ]]; then
        local current_dir="$PWD"
        local ancestor_match=""
        
        while [[ "$current_dir" != "/" && "$current_dir" != "." ]]; do
            if [[ "$(basename "$current_dir")" == "$1" ]]; then
                ancestor_match="$current_dir"
                break
            fi
            current_dir="$(dirname "$current_dir")"
        done
        
        if [[ -n "$ancestor_match" ]]; then
            cd "$ancestor_match"
            return
        fi
    fi

    # Case 5: Database query
    local results
    results=(${(f)"$(command xneo query "$@")"})
    
    case ${#results[@]} in
        0)
            echo "x: No match found for: $*" >&2
            # Show suggestions
            local suggestions
            suggestions=(${(f)"$(command xneo query --suggest "$@" 2>/dev/null)"})
            if [[ ${#suggestions[@]} -gt 0 ]]; then
                echo "Did you mean:" >&2
                printf "  %s\n" "${suggestions[@]:0:3}" >&2
            fi
            return 1
            ;;
        1)
            cd "${results[1]}"
            ;;
        *)
            local choice
            choice=$(printf "%s\n" "${results[@]}" | fzf --height=40% --reverse --border --prompt="Select directory: ")
            if [[ -n "$choice" ]]; then
                cd "$choice"
            else
                return 1
            fi
            ;;
    esac
}

# History recording hook
__xneo_add_hook() {
    command xneo add "$PWD" &
}

# Add to precmd_functions
precmd_functions+=(__xneo_add_hook)

# Bookmark alias
xb() {
    command xneo bookmark "$@"
}

# Completion for x command
_x_completion() {
    local context state line
    local suggestions
    
    _arguments \
        '*:directory:->directories'
    
    case $state in
        directories)
            suggestions=(${(f)"$(command xneo query --suggest "${words[CURRENT]}" 2>/dev/null)"})
            _describe 'directories' suggestions
            ;;
    esac
}

compdef _x_completion x
"#;
