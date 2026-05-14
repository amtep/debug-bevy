follower-type-priest = { $count ->
    [one] Priest
    *[other] Priests
}
follower-type-goon = { $count ->
    [one] Goon
    *[other] Goons
}
follower-type-minion = { $count ->
    [one] Minion
    *[other] Minions
}

follower-list-tooltip = { $count } { $follower-type ->
    *[priest] { follower-type-priest }
    [goon] { follower-type-goon }
    [minion] { follower-type-minion }
}

follower-count = { BIGNUM($count, lower-limit: 1000) }
