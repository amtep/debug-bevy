modifier = { $op ->
    *[add] { modifer-add }
    [mult] { modifier-mult }
} { modifier-shown } { modifier-duration }

modifier-add = { SIGN($amount) ->
   *[positive] +{ $amount }
    [negative]  { $amount }
    [zero] \u0020{ $amount }
}

modifier-mult = { SIGN($percent) ->
   *[positive] +{ $percent }
    [negative] { $percent }
    [zero] \u0020{ $amount }
}%

modifier-shown = { $modifier ->
   *[none] {""}
    [income] { modifier-income }
    [expense] { modifier-expense }
    [income-category] { modifier-income-category }
    [expense-category] { modifier-expense-category }
    [recruit-by] { modifier-recruit-by }
    [recruit-of] { modifier-recruit-of }
    [recruit-by-of] { modifier-recruit-by-of }
    [intelligence-suspicion] { modifier-intelligence-suspicion }
    [scientific-suspicion] { modifier-scientific-suspicion }
    [police-suspicion] { modifier-police-suspicion }
    [media-suspicion] { modifier-media-suspicion }
}

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
