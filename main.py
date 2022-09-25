


import json

if __name__ == "__main__":
    with open("/home/antonguzun/Work/personal/RePoE/RePoE/data/mods.json", "r") as f:
        data = json.load(f)
    
    options = []
    
    sum_weights = 0
    target = "helmet"
    add_filter = "base_maximum_life"
    for k in data.keys():
        if data[k]["domain"] == "item":
            for sw in data[k]["spawn_weights"]:
                if sw["tag"] == target or sw["tag"] == "default" and sw["weight"] != 0:
                    sum_weights += sw["weight"]
                if any(True if add_filter in s["id"] else False for s in data[k]["stats"]):
                    options.append({k: {"stats": data[k]["stats"], "weight": sw["weight"]}})

    k = (f"{option['id']}: {option['min']-{option['max']}}; weight: {option['weight']} ({option['weight'] / sum_weights})"
     for option in options)
    print(*k)
    print(f"total weight {sum_weights}")




# Rarity: Magic
# Crafted Item
# Iron Hat
# --------
# Quality: +20% (augmented)
# Armour: 10
# --------
# Requirements:
# Str: 9
# --------
# Item Level: 83
# --------
# +17 to maximum Life
# 19% increased Rarity of Items found
# --------
