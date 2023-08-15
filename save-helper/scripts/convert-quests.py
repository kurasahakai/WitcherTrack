import pandas as pd

df = pd.read_xml('Quests.xml')
df = df[df['type'].notnull()]
df = df[df['GUID'].notnull()]
df.to_json('quests.json', orient='records')

quest_guids = '\n'.join(sorted(set(df['GUID'])))
with open('quest-guids.txt', 'w') as fp:
    fp.write(quest_guids)

print(len(df))
