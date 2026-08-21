from BaseClasses import Tutorial
from worlds.AutoWorld import WebWorld


class SuikapelagameWebWorld(WebWorld):
    game = "Suikapelagame"

    theme = "grassFlowers"

    setup_en = Tutorial(
        "Multiworld Setup Guide",
        "A guide to setting up Suikapelagame for MultiWorld.",
        "English",
        "setup_en.md",
        "setup/en",
        ["GodOfProgramming"],
    )

    tutorials = [setup_en]
