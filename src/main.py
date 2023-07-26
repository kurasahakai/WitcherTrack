from difflib import get_close_matches
from time import sleep
import pandas as pd
import numpy as np
import pyscreenshot
import pytesseract
import datetime
import json
import cv2
import re

if __name__ == '__main__':
    #first timestamp: needed to log every 5 mins
    timestamp = datetime.datetime.now()

    #reading the lists to complete
    df = pd.read_csv("..\\data\\TW3_questlist.csv")
    quests = [a.lower() for a in df.Questname]

    df_alch = pd.read_csv("..\\data\\TW3_alchemylist.csv")
    alchemy = [a.lower() for a in df_alch.Name]
    default_alchemy = [a.lower() for a in df_alch.dropna().Name]

    df_diagrams = pd.read_csv("..\\data\\TW3_diagramlist.csv")
    crafting = [a.lower() for a in df_diagrams.Name]
    default_crafting = [a.lower() for a in df_diagrams.dropna().Name]

    #These are the types of items needed, these MUST match in some form the prompt that appears on screen
    types_of_entities = ["quest completed", "new alchemy formula", "new crafting diagram"]

    all_data = {}
    all_data["quest completed"] = quests
    all_data['new alchemy formula'] = alchemy
    all_data["new crafting diagram"] = crafting

    #Default counters
    quest_counter = 1 #Because Kaer Morhen
    alchemy_counter = len(default_alchemy)+1 #15 because Reinald's Philter is fucked + default recipes
    crafting_counter = len(default_crafting) #54 because of default recipes

    #initial writing of the files - resets data on the files when running the script
    with open("..\\output\\quest_total.txt", "w") as f:
        f.write("{}/388\n{}%".format(quest_counter, round(quest_counter / 388*100, 2)))

    with open("..\\output\\alchemy_total.txt", "w") as f:
        f.write("{}/178\n{}%".format(alchemy_counter, round(alchemy_counter / 178*100, 2)))

    with open("..\\output\\crafting_total.txt", "w") as f:
        f.write("{}/411\n{}%".format(crafting_counter, round(crafting_counter / 411*100, 2)))

    with open("..\\output\\log.txt", "w") as log_file:
        time = datetime.datetime.now().strftime('%H:%M:%S.%f')[:-4]
        log_file.write("{} - New run starting \n".format(time))

    with open("..\\output\\log_completo.txt", "w") as log:
        time = datetime.datetime.now().strftime('%H:%M:%S.%f')[:-4]
        log.write("{} - New run starting \n".format(time))

    #creating dictionaries to store the tracker status
    quests_dict = dict((quest, "NOT COMPLETED") for quest in quests)
    alchemy_dict = dict((formula, "NOT FOUND") for formula in alchemy)
    crafting_dict = dict((diagram, "NOT FOUND") for diagram in crafting)

    #EXCEPTIONS for default item given
    quests_dict["kaer morhen"] = "COMPLETED"
    for default_alc in default_alchemy:
        alchemy_dict[default_alc] = "FOUND"
    for default_craf in default_crafting:
        crafting_dict[default_craf] = "FOUND"

    #BIG ASS CONTINUOUS LOOP
    while True:
        pytesseract.pytesseract.tesseract_cmd = r'C:\Program Files\Tesseract-OCR\tesseract.exe' #Initializing tesseract

        pic = pyscreenshot.grab(bbox=(50, 500, 850, 850)) #Take a screenshot

        gray_pic = cv2.cvtColor(np.array(pic), cv2.COLOR_BGR2GRAY) #Turn it BnW

        threshold = 140 #Minimum threshold to be recognized
        assignvalue = 255  # Value to assign the pixel if the threshold is met
        threshold_method = cv2.THRESH_BINARY

        _, result = cv2.threshold(gray_pic, threshold, assignvalue, threshold_method) #Create binary mask

        #cv2.imwrite("BnW.png", result) #debug

        text = pytesseract.image_to_string(result) #detect text
        text = re.sub(" +", " ",re.sub("\n", " ", text)).lower() #remove multi spaces and newlines
        print(text)

        type = get_close_matches(text[:30], types_of_entities)  # find if the text matches one of the entity tipes

        with open("..\\output\\log_completo.txt", "a") as log:
            time = datetime.datetime.now().strftime('%H:%M:%S.%f')[:-4]
            log.write("{} - {}\n".format(time, text))
            if type:
                log.write(f"type of text:{type[0]}\n")

        if type:
            if type[0] == "quest completed":
                most_similar_list = get_close_matches(text[15:], all_data[type[0]]) #take the most similar item
            else:
                most_similar_list = get_close_matches(text, all_data[type[0]])  # take the most similar item
            if most_similar_list: #if the most similar item exists
                to_track = most_similar_list[0]  #the first item is the correct one
                print(to_track)

                with open("..\\output\\log_completo.txt", "a") as log:
                    time = datetime.datetime.now().strftime('%H:%M:%S.%f')[:-4]
                    log.write("list of similarities: {}\n".format(most_similar_list))

                #actually tracking the completion
                if type[0] == "quest completed":
                    if quests_dict[to_track] == "NOT COMPLETED":
                        quest_counter += 1
                        quests_dict[to_track] = "COMPLETED" #change status to completed
                        to_track = type[0] + ": " + to_track
                        with open("..\\output\\quest_total.txt", "w") as f:
                            f.write("{}/388\n{}%".format(quest_counter, round(quest_counter/388*100, 2))) #overlay txt file
                        print("found:", to_track)
                        # logging
                        with open("..\\output\\log.txt", "a") as log_file:
                            time = datetime.datetime.now().strftime('%H:%M:%S.%f')[:-4]
                            log_file.write("{} - {}\n".format(time, to_track))



                elif type[0] == 'new alchemy formula':
                    if alchemy_dict[to_track] == "NOT FOUND":
                        alchemy_counter += 1
                        alchemy_dict[to_track] = "FOUND" #change status to found
                        to_track = type[0] + ": " + to_track
                        with open("..\\output\\alchemy_total.txt", "w") as f:
                            f.write("{}/178\n{}%".format(alchemy_counter, round(alchemy_counter/178*100, 2))) #overlay txt file
                        print("found:", to_track)
                        # logging
                        with open("..\\output\\log.txt", "a") as log_file:
                            time = datetime.datetime.now().strftime('%H:%M:%S.%f')[:-4]
                            log_file.write("{} - {}\n".format(time, to_track))

                elif type[0] == "new crafting diagram":
                    if crafting_dict[to_track] == "NOT FOUND":
                        crafting_counter += 1
                        crafting_dict[to_track] = "FOUND" #change status to found
                        to_track = type[0] + ": " + to_track
                        with open("..\\output\\crafting_total.txt", "w") as f:
                            f.write("{}/411\n{}%".format(crafting_counter, round(crafting_counter/411*100, 2))) #overlay txt file
                        print("found:", to_track)
                        # logging
                        with open("..\\output\\log.txt", "a") as log_file:
                            time = datetime.datetime.now().strftime('%H:%M:%S.%f')[:-4]
                            log_file.write("{} - {}\n".format(time, to_track))



        time = datetime.datetime.now()
        diff = time-timestamp

        #every 5 minutes log the whole status
        if diff.seconds > 300:
            out_dict = {"quests": quests_dict, "alchemy": alchemy_dict, "crafting": crafting_dict}
            with open("..\\output\\completion.json", "w") as f:
                json.dump(out_dict, f)
            timestamp = time

        sleep(1)