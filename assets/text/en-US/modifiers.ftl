modifier-income-mult = { SIGN($percent) ->
 *[positive] { $percent }% income bonus
  [negative] { $percent }% income penalty
  [zero] no income bonus
}

modifier-expense-mult = { SIGN($percent) ->
 *[positive] { $percent }% expense increase
  [negative] { $percent }% expense reduction
  [zero] no expense modifier
}
