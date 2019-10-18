vt_list = [f.split('\t') for f in open("stazioni_coord.tsv").read().split("\n")[1:-1]]
names = {}
id_vt_map = {}
id_vtname_map = {}

def encode(sid):
    tot = 0
    for i,c in enumerate(sid):
        tot = tot + ((ord(c)-ord('A')) << i*5)
    return tot
# tmp={}
for station in vt_list:
    if station[1] in names:
        if len(station[0]) > len(names[station[1]]):
            names[station[1]] = station[0]
    else:
        names[station[1]] = station[0]

for vt_id, name in names.items():
    name_rep = name.upper().replace('-',' ').replace("`", '').replace('.', ' ').replace('  ',' ').replace('/',' ').split(' ')
    name_rep = [piece for piece in name_rep if piece != '']
    #print(name)
    first_name_len = max(1, 4-len(name_rep))
    station_id = name_rep[0][0] + (name_rep[0][-first_name_len+1:] if first_name_len>1 else '')
    # tmp[vt_id] = station_id+"\t"+str(first_name_len)
    for piece in name_rep[1:]:
        station_id = station_id + piece[0]
    #name_rep.sort(key=len, reverse=True)
    station_id = (station_id[:3] + sorted(name_rep, key=len, reverse=True)[0][-min(4, len(station_id))::-1])[:3]
    iter_num = 1
    while station_id in id_vt_map and iter_num < len(name_rep[-1]):
        station_id = station_id[:-1] + name_rep[-1][-iter_num]
        iter_num = iter_num + 1
    while station_id in id_vt_map:
        for letter in "ABCDEFGHIJKLMNOPQRSTUVWXYZ":
            for replacement in [station_id[:-1]+letter, station_id[0]+letter+station_id[2:], station_id[:-2]+2*letter, letter+station_id[1:]]:
                if not replacement in id_vt_map:
                    station_id = replacement
    id_vt_map[station_id] = vt_id
    id_vtname_map[station_id] = name
    #print(station_id[:4], first_name_len, name_rep[0], sorted(name_rep, key=len, reverse=True)[0][-min(4, len(station_id)):])

for s_id, name in id_vtname_map.items():
    print(name+"\t"+s_id + "\t" + str(encode(s_id)))# + tmp[id_vt_map[s_id]])