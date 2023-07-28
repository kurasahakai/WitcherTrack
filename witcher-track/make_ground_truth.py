import os
from glob import glob
from pathlib import Path
from tqdm import tqdm
from PIL import Image, ImageDraw, ImageFont, ImageOps

if __name__ == '__main__':
    samples = []

    for f in glob('data/*.txt'):
        with open(f, 'rt') as fp:
            samples.extend(fp.readlines())

    samples = sorted(set(
        i.strip() 
        for i in samples 
        if i.strip() != ''
    ))
    samples_words = sorted(set(
        j 
        for i in samples 
        for j in i.split() 
        if j.strip() != ''
    ))

    print(len(samples))
    print(len(samples_words))

    base_path = Path(__file__).parent / 'target' / 'ground_truth'
    os.makedirs(base_path, exist_ok=True)

    font = ImageFont.truetype('font.ttf', size=64)

    def make_ground_truth(path, text):
        box = font.getbbox(text)
        img = Image.new('L', (box[2] + 4, box[3] + 4))
        draw = ImageDraw.Draw(img)
        draw.multiline_text((2, 2), text, font=font, fill=255)
        img = ImageOps.invert(img)
        img.save(base_path / (path + '.png'))
        with open(base_path / (path + '.gt.txt'), 'w') as fp:
            fp.write(text)

    for (idx, sample) in tqdm(enumerate(samples)):
        make_ground_truth(f'sample{idx:04}', sample)

    for (idx, sample) in tqdm(enumerate(samples_words)):
        make_ground_truth(f'sample_word{idx:04}', sample)
