from collections.abc import Mapping
from typing import Any, Final
from os import linesep
import dataclasses

from BaseClasses import Region, MultiWorld
from Options import OptionError
from worlds.AutoWorld import World

from . import items, rules
from .cfg import BASE_FRUIT_ORDER
from .items import ITEM_NAME_TO_ID
from .locations import LOCATION_NAME_TO_ID, ScoresanityThreshhold, thresh_generator
from .options import SuikapelagameOptions, Goal, Scoresanity, ShuffleFruitOrder
from .web_world import SuikapelagameWebWorld


class SuikapelagameWorld(World):
    """
    Suikapelagame is a clone of Suika Game with extra features

    Taken from Suikapelago (https://github.com/Starwort/suikapelago) and rewritten in Rust + Bevy
    """

    game = "Suikapelagame"

    web = SuikapelagameWebWorld()

    options_dataclass = SuikapelagameOptions
    options: Final[SuikapelagameOptions] = SuikapelagameOptions  # type: ignore[override]

    location_name_to_id = LOCATION_NAME_TO_ID
    item_name_to_id = ITEM_NAME_TO_ID

    origin_region_name = "Game"

    def __init__(self, mw: MultiWorld, player: int):
        super().__init__(mw, player)
        self.fruit_order = BASE_FRUIT_ORDER
        self.num_locations = 0

    def generate_early(self):
        if self.options.shuffle_fruit_order:
            self.random.shuffle(self.fruit_order)

        if (
            self.options.goal != Goal.option_make_largest_fruit
            and self.options.scoresanity == Scoresanity.NO
        ):
            raise OptionError(
                "Goal requires scoresanity but scoresanity is not enabled"
            )

        if self.options.trap_chance + self.options.bonus_points_chance > 100:
            raise OptionError("Trap chance + bonus points chance is greater than 100%")

        self.num_locations = (
            len(self.fruit_order) + self.options.scoresanity.check_count()
        )

        self.min_num_items = (
            len(self.fruit_order)
            - self.options.starting_max_fruit_size
            + self.options.height_upgrade_count
            + self.options.cooldown_upgrade_count
            + self.options.next_needs_unlock
        )

        if self.min_num_items > self.num_locations:

            extra_help = []

            match self.options.scoresanity.value:
                case Scoresanity.NO:
                    extra_help.append("Try turning on scoresanity")
                case Scoresanity.YES:
                    extra_help.append(
                        "Try turning on the 'with extra' option for scoresanity"
                    )

            if self.options.starting_max_fruit_size != len(self.fruit_order):
                extra_help.append("Try increasing your max fruit size")

            if (
                self.options.height_upgrade_count.value
                == self.options.height_upgrade_count.range_end
            ):
                extra_help.append("Try decreasing the amount of height upgrades")

            if (
                self.options.cooldown_upgrade_count.value
                == self.options.cooldown_upgrade_count.range_end
            ):
                extra_help.append("Try decreasing the amount of cooldown upgrades")

            raise OptionError(
                "Current settings would generate more items than possible locations!"
                + linesep.join(extra_help)
            )

    def create_regions(self) -> None:
        self.multiworld.regions.append(Region("Game", self.player, self.multiworld))
        locations.create_all_locations(self)

    def set_rules(self) -> None:
        rules.set_all_rules(self)

    def create_items(self) -> None:
        items.create_all_items(self)

    def create_item(self, name: str) -> items.SuikapelagameItem:
        return items.create_item_with_correct_classification(self, name)

    def get_filler_item_name(self) -> str:
        return items.get_random_filler_item_name(self)

    def fill_slot_data(self) -> Mapping[str, Any]:
        return {
            "deathlink": bool(self.options.death_link),
            "shuffle_fruit": (
                self.fruit_order if self.options.shuffle_fruit_order != 0 else None
            ),
            "also_shuffle_sizes": self.options.shuffle_fruit_order
            == ShuffleFruitOrder.YES,
            "height_upgrade_count": +self.options.height_upgrade_count,
            "cooldown_upgrade_count": +self.options.cooldown_upgrade_count,
            "starting_cooldown": +self.options.starting_drop_cooldown,
            "goal": +self.options.goal,
            "scoresanity_thresholds": self.generate_scoresanity(),
            "difficulty": +self.options.scoresanity_difficulty,
            "max_fruit_size": +self.options.starting_max_fruit_size,
            "next_needs_unlock": bool(self.options.next_needs_unlock),
        }

    def generate_scoresanity(self) -> list[dict]:
        if bool(self.options.scoresanity):
            return list(
                map(
                    lambda t: dataclasses.asdict(t),
                    sorted(
                        [
                            t
                            for i in range(0, 20)
                            for t in thresh_generator(
                                i,
                                self.options.scoresanity_difficulty.value,
                                self.options.scoresanity == Scoresanity.WITH_EXTRA,
                            )
                        ],
                        key=lambda t: t.value,
                    ),
                )
            )

        return []
