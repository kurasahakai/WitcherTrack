using System;
using System.IO;
using Newtonsoft.Json;

using SaveFile;

class Quest
{
    public string Guid { get; set; }
    public string Status { get; set; }
}

public class Program
{
    static void Main(string[] Args)
    {
        var savefile = new Witcher3SaveFile("../witcher-save/QuickSave.sav", Witcher3ReadLevel.Quick);
        var quests = savefile.CJournalManager.Statuses
          .ConvertAll(status => new Quest { 
              Guid = status.PrimaryGUID, 
              Status = status.Status.ToString() 
          });
        string jsonData = JsonConvert.SerializeObject(quests, Formatting.Indented);
        File.WriteAllText("save.json", jsonData);
    }
}
