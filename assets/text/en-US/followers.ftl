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

follower-count = { $count }
follower-type-count = { $count }x { $follower-type }
follower-transfer = Transfer
follower-transfer-tooltip = move some to a another hideout

follower-transfer-current-follower-count = Current follower count: { follower-count }
follower-transfer-maximum-follower-count = Maximum follower count: { follower-count }
follower-transfer-source-base = Source hideout
follower-transfer-full-base = Full hideout
follower-transfer-number = { $follower-type } to transfer:

follower-transfer-title = Transfer { $follower-type }
follower-transfer-confirm = Go
follower-transfer-confirm-tooltip = select a destination hideout first!
follower-transfer-confirm-funds-tooltip = not enough funds, { FUNDS($funds) } required!

new-follower-toast = Recruited { $count } { $follower-type } in { $region }
