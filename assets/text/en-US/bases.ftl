basetype-apartment = Apartment
basetype-old-farmhouse = Old Farmhouse
basetype-abandoned-hospital = Abandoned Hospital
basetype-castle = Castle

menu-region-bases = Bases

acquire-apartment = Rent apartment
acquire-old-farmhouse = Buy old farmhouse
acquire-abandoned-hospital = Occupy abandoned hospital
acquire-castle = Buy castle
acquire-ballroom = Construct ballroom

acquire-tooltip-no-vacant-base-plot = there is no vacant base plot in the region!
acquire-apartment-tooltip = expand your base of operation from the confines of the urban jungle
acquire-old-farmhouse-tooltip = plot under the shadow of falling roof and the smell of manure
acquire-abandoned-hospital-tooltip = enjoy the haunted fluorescent corridors
acquire-castle-tooltip = gather your followers in the stone grand hall
acquire-ballroom-tooltip = all that glitters is not gold, guilded tombs do worms enfold

acquire-basetype-dialog-max-pop = Max follower count: { $count }
acquire-basetype-dialog-initial-cost = Initial cost: { FUNDS($funds) }
acquire-basetype-dialog-cost-per-day = Cost per day: { FUNDS($funds) }
acquire-basetype-dialog-police-suspicion = Police suspicion: { $suspicion }
acquire-basetype-dialog-media-suspicion = Media suspicion: { $suspicion }
acquire-basetype-dialog-confirm-tooltip = not enough funds, { FUNDS($funds) } required!

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
