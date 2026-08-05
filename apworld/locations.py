from __future__ import annotations

import math

from dataclasses import dataclass
from typing import TYPE_CHECKING

from BaseClasses import Location

from . import items

if TYPE_CHECKING:
    from . import SuikapelagameWorld


def extra_score_key(i: int, j: int) -> str:
    return f"Extra score threshold {i}-{j}"


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

BEST_FRUIT_IDS = {loc: i for i, loc in enumerate(BEST_FRUIT_LOCS, start=1)}
SCORE_THRESH_IDS = {
    f"Score threshold {i}": len(BEST_FRUIT_IDS) + i for i in range(1, 21)
}
EXTRA_SCORE_THRESH_IDS = {
    location_name: len(BEST_FRUIT_IDS) + len(SCORE_THRESH_IDS) + i
    for i, location_name in enumerate(
        (extra_score_key(i, j) for i in range(5, 20) for j in range(1, i // 5 + 1)),
        start=1,
    )
}

LOCATION_NAME_TO_ID = {
    **BEST_FRUIT_IDS,
    **SCORE_THRESH_IDS,
    **EXTRA_SCORE_THRESH_IDS,
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


@dataclass
class ScoresanityThreshhold:
    value: int
    location: int


def thresh_generator(t: int, difficulty: int, with_extra: bool):
    value = int((1500 + 500 * difficulty) * pow((t + 1) / 20, 2.5))
    location = 12 + t

    yield ScoresanityThreshhold(value=value, location=location)

    if with_extra and t >= 5:
        intervals = math.floor(t / 5)
        n = next(thresh_generator(t + 1, difficulty, False))
        interval_value = int((n.value - value) / (intervals + 1))
        for i in range(1, intervals + 1):
            key = extra_score_key(t, i)
            location = LOCATION_NAME_TO_ID[key]
            assert location is not None
            yield ScoresanityThreshhold(
                value=value + interval_value * i, location=location
            )
