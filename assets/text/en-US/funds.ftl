funds-display = { FUNDS($funds) }
funds = { FUNDS($funds) }
funds-change-display = { FUNDS($funds, max-dp: 1, lower-limit: 10, sign: "blank") }

income-tooltip-header = Income
expense-tooltip-header = Expenses

income-category-global = { $count ->
    [one] Modifier
    *[other] Modifiers
}

income-category-job = { $count ->
    [one] Job
    *[other] Jobs
}
income-category-crime = Crime

income-category-event = { $count ->
    [one] Event
    *[other] Events
}

expense-category-global = { $count ->
    [one] Modifier
    *[other] Modifiers
}

expense-category-base = { $count ->
    [one] Hideout
    *[other] Hideouts
}
expense-category-research = Research

expense-category-event = { $count ->
    [one] Event
    *[other] Events
}

expense-category-priest = { follower-type-priest }
expense-category-minion = { follower-type-minion }
expense-category-goon = { follower-type-goon }
