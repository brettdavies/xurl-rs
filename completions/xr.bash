_xr() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="xr"
                ;;
            xr,auth)
                cmd="xr__subcmd__auth"
                ;;
            xr,bookmark)
                cmd="xr__subcmd__bookmark"
                ;;
            xr,bookmarks)
                cmd="xr__subcmd__bookmarks"
                ;;
            xr,completions)
                cmd="xr__subcmd__completions"
                ;;
            xr,delete)
                cmd="xr__subcmd__delete"
                ;;
            xr,dm)
                cmd="xr__subcmd__dm"
                ;;
            xr,dms)
                cmd="xr__subcmd__dms"
                ;;
            xr,examples)
                cmd="xr__subcmd__examples"
                ;;
            xr,follow)
                cmd="xr__subcmd__follow"
                ;;
            xr,followers)
                cmd="xr__subcmd__followers"
                ;;
            xr,following)
                cmd="xr__subcmd__following"
                ;;
            xr,help)
                cmd="xr__subcmd__help"
                ;;
            xr,like)
                cmd="xr__subcmd__like"
                ;;
            xr,likes)
                cmd="xr__subcmd__likes"
                ;;
            xr,media)
                cmd="xr__subcmd__media"
                ;;
            xr,mentions)
                cmd="xr__subcmd__mentions"
                ;;
            xr,mute)
                cmd="xr__subcmd__mute"
                ;;
            xr,post)
                cmd="xr__subcmd__post"
                ;;
            xr,quote)
                cmd="xr__subcmd__quote"
                ;;
            xr,read)
                cmd="xr__subcmd__read"
                ;;
            xr,reply)
                cmd="xr__subcmd__reply"
                ;;
            xr,repost)
                cmd="xr__subcmd__repost"
                ;;
            xr,schema)
                cmd="xr__subcmd__schema"
                ;;
            xr,search)
                cmd="xr__subcmd__search"
                ;;
            xr,skill)
                cmd="xr__subcmd__skill"
                ;;
            xr,timeline)
                cmd="xr__subcmd__timeline"
                ;;
            xr,unbookmark)
                cmd="xr__subcmd__unbookmark"
                ;;
            xr,unfollow)
                cmd="xr__subcmd__unfollow"
                ;;
            xr,unlike)
                cmd="xr__subcmd__unlike"
                ;;
            xr,unmute)
                cmd="xr__subcmd__unmute"
                ;;
            xr,unrepost)
                cmd="xr__subcmd__unrepost"
                ;;
            xr,usage)
                cmd="xr__subcmd__usage"
                ;;
            xr,user)
                cmd="xr__subcmd__user"
                ;;
            xr,validate)
                cmd="xr__subcmd__validate"
                ;;
            xr,version)
                cmd="xr__subcmd__version"
                ;;
            xr,whoami)
                cmd="xr__subcmd__whoami"
                ;;
            xr__subcmd__auth,app)
                cmd="xr__subcmd__auth__subcmd__app"
                ;;
            xr__subcmd__auth,apps)
                cmd="xr__subcmd__auth__subcmd__apps"
                ;;
            xr__subcmd__auth,clear)
                cmd="xr__subcmd__auth__subcmd__clear"
                ;;
            xr__subcmd__auth,default)
                cmd="xr__subcmd__auth__subcmd__default"
                ;;
            xr__subcmd__auth,help)
                cmd="xr__subcmd__auth__subcmd__help"
                ;;
            xr__subcmd__auth,oauth1)
                cmd="xr__subcmd__auth__subcmd__oauth1"
                ;;
            xr__subcmd__auth,oauth2)
                cmd="xr__subcmd__auth__subcmd__oauth2"
                ;;
            xr__subcmd__auth,status)
                cmd="xr__subcmd__auth__subcmd__status"
                ;;
            xr__subcmd__auth__subcmd__apps,add)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__add"
                ;;
            xr__subcmd__auth__subcmd__apps,help)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__help"
                ;;
            xr__subcmd__auth__subcmd__apps,list)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__list"
                ;;
            xr__subcmd__auth__subcmd__apps,redirect-uri)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri"
                ;;
            xr__subcmd__auth__subcmd__apps,remove)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__remove"
                ;;
            xr__subcmd__auth__subcmd__apps,update)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__update"
                ;;
            xr__subcmd__auth__subcmd__apps__subcmd__help,add)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__add"
                ;;
            xr__subcmd__auth__subcmd__apps__subcmd__help,help)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__help"
                ;;
            xr__subcmd__auth__subcmd__apps__subcmd__help,list)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__list"
                ;;
            xr__subcmd__auth__subcmd__apps__subcmd__help,redirect-uri)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect__subcmd__uri"
                ;;
            xr__subcmd__auth__subcmd__apps__subcmd__help,remove)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__remove"
                ;;
            xr__subcmd__auth__subcmd__apps__subcmd__help,update)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__update"
                ;;
            xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect__subcmd__uri,get)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect__subcmd__uri__subcmd__get"
                ;;
            xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect__subcmd__uri,set)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect__subcmd__uri__subcmd__set"
                ;;
            xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri,get)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__get"
                ;;
            xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri,help)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__help"
                ;;
            xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri,set)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__set"
                ;;
            xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__help,get)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__help__subcmd__get"
                ;;
            xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__help,help)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__help__subcmd__help"
                ;;
            xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__help,set)
                cmd="xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__help__subcmd__set"
                ;;
            xr__subcmd__auth__subcmd__help,app)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__app"
                ;;
            xr__subcmd__auth__subcmd__help,apps)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__apps"
                ;;
            xr__subcmd__auth__subcmd__help,clear)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__clear"
                ;;
            xr__subcmd__auth__subcmd__help,default)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__default"
                ;;
            xr__subcmd__auth__subcmd__help,help)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__help"
                ;;
            xr__subcmd__auth__subcmd__help,oauth1)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__oauth1"
                ;;
            xr__subcmd__auth__subcmd__help,oauth2)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__oauth2"
                ;;
            xr__subcmd__auth__subcmd__help,status)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__status"
                ;;
            xr__subcmd__auth__subcmd__help__subcmd__apps,add)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__add"
                ;;
            xr__subcmd__auth__subcmd__help__subcmd__apps,list)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__list"
                ;;
            xr__subcmd__auth__subcmd__help__subcmd__apps,redirect-uri)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect__subcmd__uri"
                ;;
            xr__subcmd__auth__subcmd__help__subcmd__apps,remove)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__remove"
                ;;
            xr__subcmd__auth__subcmd__help__subcmd__apps,update)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__update"
                ;;
            xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect__subcmd__uri,get)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__get"
                ;;
            xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect__subcmd__uri,set)
                cmd="xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__set"
                ;;
            xr__subcmd__help,auth)
                cmd="xr__subcmd__help__subcmd__auth"
                ;;
            xr__subcmd__help,bookmark)
                cmd="xr__subcmd__help__subcmd__bookmark"
                ;;
            xr__subcmd__help,bookmarks)
                cmd="xr__subcmd__help__subcmd__bookmarks"
                ;;
            xr__subcmd__help,completions)
                cmd="xr__subcmd__help__subcmd__completions"
                ;;
            xr__subcmd__help,delete)
                cmd="xr__subcmd__help__subcmd__delete"
                ;;
            xr__subcmd__help,dm)
                cmd="xr__subcmd__help__subcmd__dm"
                ;;
            xr__subcmd__help,dms)
                cmd="xr__subcmd__help__subcmd__dms"
                ;;
            xr__subcmd__help,examples)
                cmd="xr__subcmd__help__subcmd__examples"
                ;;
            xr__subcmd__help,follow)
                cmd="xr__subcmd__help__subcmd__follow"
                ;;
            xr__subcmd__help,followers)
                cmd="xr__subcmd__help__subcmd__followers"
                ;;
            xr__subcmd__help,following)
                cmd="xr__subcmd__help__subcmd__following"
                ;;
            xr__subcmd__help,help)
                cmd="xr__subcmd__help__subcmd__help"
                ;;
            xr__subcmd__help,like)
                cmd="xr__subcmd__help__subcmd__like"
                ;;
            xr__subcmd__help,likes)
                cmd="xr__subcmd__help__subcmd__likes"
                ;;
            xr__subcmd__help,media)
                cmd="xr__subcmd__help__subcmd__media"
                ;;
            xr__subcmd__help,mentions)
                cmd="xr__subcmd__help__subcmd__mentions"
                ;;
            xr__subcmd__help,mute)
                cmd="xr__subcmd__help__subcmd__mute"
                ;;
            xr__subcmd__help,post)
                cmd="xr__subcmd__help__subcmd__post"
                ;;
            xr__subcmd__help,quote)
                cmd="xr__subcmd__help__subcmd__quote"
                ;;
            xr__subcmd__help,read)
                cmd="xr__subcmd__help__subcmd__read"
                ;;
            xr__subcmd__help,reply)
                cmd="xr__subcmd__help__subcmd__reply"
                ;;
            xr__subcmd__help,repost)
                cmd="xr__subcmd__help__subcmd__repost"
                ;;
            xr__subcmd__help,schema)
                cmd="xr__subcmd__help__subcmd__schema"
                ;;
            xr__subcmd__help,search)
                cmd="xr__subcmd__help__subcmd__search"
                ;;
            xr__subcmd__help,skill)
                cmd="xr__subcmd__help__subcmd__skill"
                ;;
            xr__subcmd__help,timeline)
                cmd="xr__subcmd__help__subcmd__timeline"
                ;;
            xr__subcmd__help,unbookmark)
                cmd="xr__subcmd__help__subcmd__unbookmark"
                ;;
            xr__subcmd__help,unfollow)
                cmd="xr__subcmd__help__subcmd__unfollow"
                ;;
            xr__subcmd__help,unlike)
                cmd="xr__subcmd__help__subcmd__unlike"
                ;;
            xr__subcmd__help,unmute)
                cmd="xr__subcmd__help__subcmd__unmute"
                ;;
            xr__subcmd__help,unrepost)
                cmd="xr__subcmd__help__subcmd__unrepost"
                ;;
            xr__subcmd__help,usage)
                cmd="xr__subcmd__help__subcmd__usage"
                ;;
            xr__subcmd__help,user)
                cmd="xr__subcmd__help__subcmd__user"
                ;;
            xr__subcmd__help,validate)
                cmd="xr__subcmd__help__subcmd__validate"
                ;;
            xr__subcmd__help,version)
                cmd="xr__subcmd__help__subcmd__version"
                ;;
            xr__subcmd__help,whoami)
                cmd="xr__subcmd__help__subcmd__whoami"
                ;;
            xr__subcmd__help__subcmd__auth,app)
                cmd="xr__subcmd__help__subcmd__auth__subcmd__app"
                ;;
            xr__subcmd__help__subcmd__auth,apps)
                cmd="xr__subcmd__help__subcmd__auth__subcmd__apps"
                ;;
            xr__subcmd__help__subcmd__auth,clear)
                cmd="xr__subcmd__help__subcmd__auth__subcmd__clear"
                ;;
            xr__subcmd__help__subcmd__auth,default)
                cmd="xr__subcmd__help__subcmd__auth__subcmd__default"
                ;;
            xr__subcmd__help__subcmd__auth,oauth1)
                cmd="xr__subcmd__help__subcmd__auth__subcmd__oauth1"
                ;;
            xr__subcmd__help__subcmd__auth,oauth2)
                cmd="xr__subcmd__help__subcmd__auth__subcmd__oauth2"
                ;;
            xr__subcmd__help__subcmd__auth,status)
                cmd="xr__subcmd__help__subcmd__auth__subcmd__status"
                ;;
            xr__subcmd__help__subcmd__auth__subcmd__apps,add)
                cmd="xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__add"
                ;;
            xr__subcmd__help__subcmd__auth__subcmd__apps,list)
                cmd="xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__list"
                ;;
            xr__subcmd__help__subcmd__auth__subcmd__apps,redirect-uri)
                cmd="xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri"
                ;;
            xr__subcmd__help__subcmd__auth__subcmd__apps,remove)
                cmd="xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__remove"
                ;;
            xr__subcmd__help__subcmd__auth__subcmd__apps,update)
                cmd="xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__update"
                ;;
            xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri,get)
                cmd="xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__get"
                ;;
            xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri,set)
                cmd="xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__set"
                ;;
            xr__subcmd__help__subcmd__media,status)
                cmd="xr__subcmd__help__subcmd__media__subcmd__status"
                ;;
            xr__subcmd__help__subcmd__media,upload)
                cmd="xr__subcmd__help__subcmd__media__subcmd__upload"
                ;;
            xr__subcmd__help__subcmd__skill,install)
                cmd="xr__subcmd__help__subcmd__skill__subcmd__install"
                ;;
            xr__subcmd__help__subcmd__skill,update)
                cmd="xr__subcmd__help__subcmd__skill__subcmd__update"
                ;;
            xr__subcmd__help__subcmd__usage,credits)
                cmd="xr__subcmd__help__subcmd__usage__subcmd__credits"
                ;;
            xr__subcmd__media,help)
                cmd="xr__subcmd__media__subcmd__help"
                ;;
            xr__subcmd__media,status)
                cmd="xr__subcmd__media__subcmd__status"
                ;;
            xr__subcmd__media,upload)
                cmd="xr__subcmd__media__subcmd__upload"
                ;;
            xr__subcmd__media__subcmd__help,help)
                cmd="xr__subcmd__media__subcmd__help__subcmd__help"
                ;;
            xr__subcmd__media__subcmd__help,status)
                cmd="xr__subcmd__media__subcmd__help__subcmd__status"
                ;;
            xr__subcmd__media__subcmd__help,upload)
                cmd="xr__subcmd__media__subcmd__help__subcmd__upload"
                ;;
            xr__subcmd__skill,help)
                cmd="xr__subcmd__skill__subcmd__help"
                ;;
            xr__subcmd__skill,install)
                cmd="xr__subcmd__skill__subcmd__install"
                ;;
            xr__subcmd__skill,update)
                cmd="xr__subcmd__skill__subcmd__update"
                ;;
            xr__subcmd__skill__subcmd__help,help)
                cmd="xr__subcmd__skill__subcmd__help__subcmd__help"
                ;;
            xr__subcmd__skill__subcmd__help,install)
                cmd="xr__subcmd__skill__subcmd__help__subcmd__install"
                ;;
            xr__subcmd__skill__subcmd__help,update)
                cmd="xr__subcmd__skill__subcmd__help__subcmd__update"
                ;;
            xr__subcmd__usage,credits)
                cmd="xr__subcmd__usage__subcmd__credits"
                ;;
            xr__subcmd__usage,help)
                cmd="xr__subcmd__usage__subcmd__help"
                ;;
            xr__subcmd__usage__subcmd__help,credits)
                cmd="xr__subcmd__usage__subcmd__help__subcmd__credits"
                ;;
            xr__subcmd__usage__subcmd__help,help)
                cmd="xr__subcmd__usage__subcmd__help__subcmd__help"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        xr)
            opts="-X -H -d -u -v -t -s -F -q -h -V --method --header --data --auth --username --verbose --trace --stream --file --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help --version post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --method)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -X)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --header)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --data)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -F)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth)
            opts="-v -q -h --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help oauth2 oauth1 app status clear apps default help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__app)
            opts="-v -q -h --bearer-token --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --bearer-token)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps)
            opts="-v -q -h --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help add update remove list redirect-uri help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__add)
            opts="-v -q -h --client-id --client-secret --redirect-uri --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --client-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --client-secret)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --redirect-uri)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__help)
            opts="add update remove list redirect-uri help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect__subcmd__uri)
            opts="get set"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect__subcmd__uri__subcmd__get)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 6 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect__subcmd__uri__subcmd__set)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 6 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__update)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__list)
            opts="-v -q -h --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri)
            opts="-v -q -h --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help get set help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__get)
            opts="-v -q -h --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__help)
            opts="get set help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__help__subcmd__get)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 6 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 6 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__help__subcmd__set)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 6 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__set)
            opts="-v -q -h --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__remove)
            opts="-v -q -h --force --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__apps__subcmd__update)
            opts="-v -q -h --client-id --client-secret --redirect-uri --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --client-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --client-secret)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --redirect-uri)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__clear)
            opts="-v -q -h --all --oauth1 --oauth2-username --bearer --force --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --oauth2-username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__default)
            opts="-v -q -h --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help)
            opts="oauth2 oauth1 app status clear apps default help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__app)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__apps)
            opts="add update remove list redirect-uri"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect__subcmd__uri)
            opts="get set"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__get)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 6 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__set)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 6 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__update)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__clear)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__default)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__oauth1)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__oauth2)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__help__subcmd__status)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__oauth1)
            opts="-v -q -h --consumer-key --consumer-secret --access-token --token-secret --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --consumer-key)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --consumer-secret)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --access-token)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --token-secret)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__oauth2)
            opts="-v -q -h --no-browser --step --auth-url --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --no-browser)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --step)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --auth-url)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__auth__subcmd__status)
            opts="-v -q -h --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__bookmark)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__bookmarks)
            opts="-n -u -t -v -q -h --max-results --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --max-results)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__completions)
            opts="-v -q -h --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help bash elvish fish powershell zsh"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__delete)
            opts="-u -t -v -q -h --force --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__dm)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__dms)
            opts="-n -u -t -v -q -h --max-results --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --max-results)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__examples)
            opts="-v -q -h --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__follow)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__followers)
            opts="-n -u -t -v -q -h --max-results --of --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --max-results)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --of)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__following)
            opts="-n -u -t -v -q -h --max-results --of --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --max-results)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --of)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help)
            opts="post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth)
            opts="oauth2 oauth1 app status clear apps default"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth__subcmd__app)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth__subcmd__apps)
            opts="add update remove list redirect-uri"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri)
            opts="get set"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__get)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 6 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect__subcmd__uri__subcmd__set)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 6 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__update)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth__subcmd__clear)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth__subcmd__default)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth__subcmd__oauth1)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth__subcmd__oauth2)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__auth__subcmd__status)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__bookmark)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__bookmarks)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__completions)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__delete)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__dm)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__dms)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__examples)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__follow)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__followers)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__following)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__like)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__likes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__media)
            opts="upload status"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__media__subcmd__status)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__media__subcmd__upload)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__mentions)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__mute)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__post)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__quote)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__read)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__reply)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__repost)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__schema)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__search)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__skill)
            opts="install update"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__skill__subcmd__install)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__skill__subcmd__update)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__timeline)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__unbookmark)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__unfollow)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__unlike)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__unmute)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__unrepost)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__usage)
            opts="credits"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__usage__subcmd__credits)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__user)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__validate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__version)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__help__subcmd__whoami)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__like)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__likes)
            opts="-n -u -t -v -q -h --max-results --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --max-results)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__media)
            opts="-v -q -h --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help upload status help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__media__subcmd__help)
            opts="upload status help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__media__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__media__subcmd__help__subcmd__status)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__media__subcmd__help__subcmd__upload)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__media__subcmd__status)
            opts="-u -w -t -H -v -q -h --auth --username --wait --trace --header --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --header)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__media__subcmd__upload)
            opts="-u -t -H -v -q -h --media-type --category --wait --auth --username --trace --header --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --media-type)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --category)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --header)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__mentions)
            opts="-n -u -t -v -q -h --max-results --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --max-results)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__mute)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__post)
            opts="-u -t -v -q -h --media-id --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --media-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__quote)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__read)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__reply)
            opts="-u -t -v -q -h --media-id --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --media-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__repost)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__schema)
            opts="-v -q -h --list --all --envelope --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__search)
            opts="-n -u -t -v -q -h --max-results --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --max-results)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__skill)
            opts="-v -q -h --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help install update help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__skill__subcmd__help)
            opts="install update help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__skill__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__skill__subcmd__help__subcmd__install)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__skill__subcmd__help__subcmd__update)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__skill__subcmd__install)
            opts="-v -q -h --all --dry-run --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --limit --cursor --page --after --help claude_code codex cursor factory kiro opencode"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__skill__subcmd__update)
            opts="-v -q -h --all --dry-run --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --limit --cursor --page --after --help claude_code codex cursor factory kiro opencode"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__timeline)
            opts="-n -u -t -v -q -h --max-results --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --max-results)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__unbookmark)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__unfollow)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__unlike)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__unmute)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__unrepost)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__usage)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help credits help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__usage__subcmd__credits)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__usage__subcmd__help)
            opts="credits help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__usage__subcmd__help__subcmd__credits)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__usage__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__user)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__validate)
            opts="-v -q -h --schema --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --schema)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__version)
            opts="-v -q -h --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        xr__subcmd__whoami)
            opts="-u -t -v -q -h --auth --username --trace --verbose --app --output --json --jsonl --raw --no-pager --quiet --no-interactive --timeout --color --dry-run --limit --cursor --page --after --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verbose)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --app)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "text json jsonl ndjson yaml csv tsv" -- "${cur}"))
                    return 0
                    ;;
                --raw)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --quiet)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -q)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --no-interactive)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --dry-run)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cursor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --page)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _xr -o nosort -o bashdefault -o default xr
else
    complete -F _xr -o bashdefault -o default xr
fi
