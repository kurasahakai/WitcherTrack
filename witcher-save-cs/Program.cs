using System;
using System.IO;
using System.Runtime.InteropServices;
using Newtonsoft.Json;

using SaveFile;

class Quest
{
    public string Guid { get; set; }
    public string Status { get; set; }
}

public class Program
{
    [UnmanagedCallersOnly(EntryPoint = "export_save")]
    public static void ExportSave(IntPtr pPath)
    {
        var path = Marshal.PtrToStringAnsi(pPath);
        var savefile = new Witcher3SaveFile(path, Witcher3ReadLevel.Quick);
        var quests = savefile.CJournalManager.Statuses
          .ConvertAll(status => new Quest { 
              Guid = status.PrimaryGUID, 
              Status = status.Status.ToString() 
          });
        string jsonData = JsonConvert.SerializeObject(quests, Formatting.Indented);
        File.WriteAllText("save.json", jsonData);
    }
}
