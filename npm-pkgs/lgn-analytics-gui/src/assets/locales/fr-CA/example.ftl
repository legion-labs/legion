# Simple things are simple.
hello-user = Allô {$userName}!

# Complex things are possible.
shared-photos =
    {$userName} {$photoCount ->
        [0] n'a ajouté aucune nouvelle photo
        [one] a ajouté une nouvelle photo
       *[other] a ajouté {$photoCount} nouvelles photos
    } à son album.
