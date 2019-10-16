from json import loads
from urllib.request import urlopen
from urllib.parse import quote_plus

mapping = {}

for row in open("vt_lf_map.tsv").read().split("\n"):
    row_vals = row.split("\t")
    mapping[row_vals[0]] = mapping[row_vals[1]]

base_url = "https://www.lefrecce.it/msite/api/geolocations/locations?name="

stations_list = [row.split("\t")[0] for row in open("stazioni_coord.tsv").read().replace("\r", '').split("\n")[1:]]

for station in stations_list:
    print(station)
    found = False
    iters = 0
    while not found:
        try:
            response = loads(urlopen(base_url + quote_plus(station.replace('`',"'")[:-iters] if iters > 0 else station)).read())
        except:
            with open("vt_lf_map.tsv", 'w') as f:
                f.write('\n'.join([k+'\t'+v for k,v in mapping.items()]))
            exit()
        #print(base_url + quote_plus(station[:-iters] if iters > 0 else station), response)
        if len(response) == 0:
            iters = iters + 1
        else:
            mapping[station] = response[0]["name"]
            found = True

with open("vt_lf_map.tsv", 'w') as f:
    f.write('\n'.join([k+'\t'+v for k,v in mapping.items()]))
