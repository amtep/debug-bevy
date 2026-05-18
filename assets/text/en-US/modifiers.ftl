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

modifier-intelligence-suspicion-mult = { SIGN($percent) ->
 *[positive] { $percent }% intelligence suspicion change increase
  [negative] { $percent }% intelligence suspicion change reduction
  [zero] no expense modifier
}

modifier-scientific-suspicion-mult = { SIGN($percent) ->
 *[positive] { $percent }% scientific suspicion change increase
  [negative] { $percent }% scientific suspicion change reduction
  [zero] no expense modifier
}

modifier-police-suspicion-mult = { SIGN($percent) ->
 *[positive] { $percent }% police suspicion change increase
  [negative] { $percent }% police suspicion change reduction
  [zero] no expense modifier
}

modifier-media-suspicion-mult = { SIGN($percent) ->
 *[positive] { $percent }% media suspicion change increase
  [negative] { $percent }% media suspicion change reduction
  [zero] no expense modifier
}
