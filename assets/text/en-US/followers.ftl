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

follower-type-name = { $follower-type ->
    *[priest] { follower-type-priest }
    [goon] { follower-type-goon }
    [minion] { follower-type-minion }
}

follower-list-tooltip = { $count } { follower-type-name }

follower-count = { BIGNUM($count, lower-limit: 1000) }

follower-transfer = Transfer
follower-transfer-tooltip = move some to a another hideout

follower-transfer-current-follower-count = Current follower count: { follower-count }
follower-transfer-maximum-follower-count = Maximum follower count: { follower-count }
follower-transfer-source-base = Source hideout
follower-transfer-full-base = Full hideout
follower-transfer-number = { follower-type-name } to transfer:

follower-transfer-title = Transfer { follower-type-name }
follower-transfer-confirm = Go
follower-transfer-confirm-tooltip = select a destination hideout first!
