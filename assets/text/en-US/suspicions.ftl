intelligence-suspicion = Intelligence suspicion
scientific-suspicion = Scientific suspicion
police-suspicion = Police suspicion
media-suspicion = Media suspicion
suspicion = { $suspicion ->
    *[intelligence] { intelligence-suspicion }
    [scientific] { scientific-suspicion }
    [police] { police-suspicion }
    [media] { media-suspicion }
}

suspicion-change = { BIGNUM($amount, lower-limit: 0, sign: "blank") }
