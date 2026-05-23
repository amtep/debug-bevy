# $op, $modifier, $amount/$percent, $duration/$date
modifier = { $op ->
    *[add] { modifer-add }
    [mult] { modifier-mult }
} { $modifier } { modifier-duration }

modifier-add = { SIGN($amount) ->
   *[positive] +{ $amount }
    [negative]  { $amount }
    [zero] \u0020{ $amount }
}

modifier-mult = { SIGN($percent) ->
   *[positive] +{ $percent }
    [negative] { $percent }
    [zero] \u0020{ $percent }
}%

modifier-duration = { $duration ->
   *[0] {""}
    [-1] ending on { DATETIME($date, dateStyle: "short") }
    [other] for { $duration } days
}

modifier-income = income
modifier-expense = expense
modifier-income-category = income for { $cat }
modifier-expense-category = expense for { $cat }
modifier-recruit-by = recruitment by { $by }
modifier-recruit-of = recruitment of { $of }
modifier-recruit-by-of = recruitment by { $by } of { $of }
modifier-intelligence-suspicion = intelligence suspicion
modifier-scientific-suspicion = scientific suspicion
modifier-police-suspicion = police suspicion
modifier-media-suspicion = media suspicion
