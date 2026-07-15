from __future__ import annotations

from typing import TYPE_CHECKING

from BaseClasses import Location

from . import items

if TYPE_CHECKING:
    from . import SuikapelagameWorld

BEST_FRUIT_LOCS = [
    "Best fruit: Cherry or better",
    "Best fruit: Strawberry or better",
    "Best fruit: Grapes or better",
    "Best fruit: Dekopon or better",
    "Best fruit: Persimmon or better",
    "Best fruit: Apple or better",
    "Best fruit: Pear or better",
    "Best fruit: Peach or better",
    "Best fruit: Pineapple or better",
    "Best fruit: Melon or better",
    "Best fruit: Watermelon or better",
]

LOCATION_NAME_TO_ID = {
    **{loc: i for i, loc in enumerate(BEST_FRUIT_LOCS, start=1)},
    **{f"Score threshold {i}": len(BEST_FRUIT_LOCS) + i for i in range(1, 21)},
    **{
        threshold: 31 + i
        for i, threshold in enumerate(
            (
                f"Extra score threshold {i}-{j}"
                for i in range(5, 20)
                for j in range(1, i // 5 + 1)
            ),
            start=1,
        )
    },
}


class SuikapelagameLocation(Location):
    game = "Suikapelagame"


def create_all_locations(world: SuikapelagameWorld) -> None:
    create_regular_locations(world)
    create_events(world)


def create_regular_locations(world: SuikapelagameWorld) -> None:
    game = world.get_region("Game")

    def is_location_enabled(location):
        name, _ = location
        return (
            "Best fruit" in name
            or ("Score" in name and world.options.scoresanity)
            or ("Extra" in name and world.options.scoresanity == 2)
        )

    game.add_locations(
        dict(filter(is_location_enabled, LOCATION_NAME_TO_ID.items())),
        SuikapelagameLocation,
    )


def create_events(world: SuikapelagameWorld) -> None:
    game = world.get_region("Game")
    game.add_event(
        "Victory",
        "Victory",
        location_type=SuikapelagameLocation,
        item_type=items.SuikapelagameItem,
    )
