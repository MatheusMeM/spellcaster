# Profiles, patch and groups. Importing the package already registers patch_list / calib_hold / calib_sweep in the registry.
from .profile import Profile, ProfileError, load, names
from .patch import Fixture, Patch, PatchError, patch_list
from .group import Group, GROUPS, calib_hold, calib_sweep

__all__ = ["Profile", "ProfileError", "load", "names", "Fixture", "Patch", "PatchError", "patch_list",
           "Group", "GROUPS", "calib_hold", "calib_sweep"]


# --- timeline "fixture" track ----------------------------------------------
# .spell: "patch":[{"name":"bsw_1","profile":"bsw_scorpio_17","universe":1,"address":300}],
#         "tracks":[{"type":"fixture","fixture":"bsw_1","dim":[[0,0],[2,255]],"color":[[0,"amarelo"]]}]
_PATCHES = {}   # ponytail: one Patch per Universes (id) ; clear it if Universes become short-lived


def _fixture(tr, t, uni):
    name = tr.spec.get("fixture")
    if not name:
        return                                                # track without a fixture: ignored
    p = _PATCHES.get(id(uni))
    if p is None:
        p = _PATCHES[id(uni)] = Patch(uni)
        for f in tr.patch:
            p.add(f["name"], f["profile"], int(f.get("universe", 1)), int(f["address"]))
    vals = {k: ks.value(t) for k, ks in tr.fparams.items()}   # ponytail: one dict per frame ; cache it if it shows up in the profiler
    p.set(name, **{k: v for k, v in vals.items() if v is not None})


from ..timeline.model import Timeline                       # noqa: E402
Timeline.resolvers["fixture"] = _fixture
