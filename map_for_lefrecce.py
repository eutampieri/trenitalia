from json import loads
from urllib.request import urlopen
from urllib.parse import quote_plus

mapping = {}
mapping_vt_id = {}

for row in open("stations.tsv").read().replace("\r", '').split("\n"):
    data = row.split("\t")
    mapping_vt_id[data[0]] = data[1]

for row in open("id_lf_map.tsv").read().split("\n"):
    row_vals = row.split("\t")
    mapping[row_vals[0]] = row_vals[1]

base_url = "https://www.lefrecce.it/msite/api/geolocations/locations?name="

stations_list = [row.split("\t") for row in open("stations.tsv").read().replace("\r", '').split("\n")]

for station in stations_list:
    name = station[0]
    print(name)
    found = station[1] in mapping
    iters = 0
    while not found:
        try:
            response = loads(urlopen(base_url + quote_plus(name.replace('`',"'")[:-iters] if iters > 0 else name)).read())
        except:
            with open("id_lf_map.tsv", 'w') as f:
                f.write('\n'.join([k+'\t'+v for k,v in mapping.items()]))
            exit()
        #print(base_url + quote_plus(station[:-iters] if iters > 0 else station), response)
        if len(response) == 0:
            iters = iters + 1
        else:
            mapping[station[1]] = response[0]["name"]
            found = True

with open("id_lf_map.tsv", 'w') as f:
    f.write('\n'.join([k+'\t'+v for k,v in mapping.items()]))
