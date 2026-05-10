funds-display = { FUNDS($funds) }
funds = { FUNDS($funds) }

income-tooltip-header = Income
expense-tooltip-header = Expenses

income-category-job = { $count ->
    [one] Job
    *[other] Jobs
}
income-category-crime = { $count ->
    [one] Crime
    *[other] Crime
}

expense-category-base = { $count ->
    [one] Base
    *[other] Bases
}

expense-category-priest = { follower-type-priest }
expense-category-minion = { follower-type-minion }
expense-category-goon = { follower-type-goon }
