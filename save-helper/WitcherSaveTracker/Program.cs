using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Text.Json;

using W3SavegameEditor.Core.Savegame;
using W3SavegameEditor.Core.Savegame.Variables;
using W3SavegameEditor.Core.Savegame.Values;
using SaveFile;

public class TrackerInfo {
    public List<Quest> quests { get; set; }
    public List<string> map_pin_tags { get; set; }
}

public class Quest
{
    public string Guid { get; set; }
    public string Status { get; set; }
}

public class Program
{
    // [UnmanagedCallersOnly(EntryPoint = "export_save")]
    // public static void ExportSave(IntPtr pPath)
    // {
    //     var path = Marshal.PtrToStringAnsi(pPath);
    //     ExportSaveInternal(path);
    // }

    static void ExportSaveInternal(string path) {
      var map_pin_tags = ExtractMapPinTags(path);
      var quests = ExtractQuests(path);

      var tracker_info = new TrackerInfo {
          map_pin_tags = map_pin_tags,
          quests = quests
      };
      var json = JsonSerializer.Serialize(tracker_info);

      File.WriteAllText("tw3trackerinfo.json", json);
    }

    static List<string> ExtractMapPinTags(string path) {
        var savegame = SavegameFile.Read(path);
        var map_pin_tags = new List<string>();

        var common_map_manager = Array.Find(
            savegame.Variables, 
            x => x.Name == "CCommonMapManager"
        ) as BsVariable;

        foreach (var v in common_map_manager.Variables) {
          if (v.Name == "MapPinTag") {
              VlVariable vl = v as VlVariable;
              map_pin_tags.Add(vl.Value.ToString());
          }
        }

        return map_pin_tags;
    }

    static List<Quest> ExtractQuests (string path) {
        var savefile = new Witcher3SaveFile(path, Witcher3ReadLevel.Quick);
        var quests = savefile.CJournalManager.Statuses
          .ConvertAll(status => new Quest { 
              Guid = status.PrimaryGUID, 
              Status = status.Status.ToString() 
          });

        return quests;
    }

    public static void Main(string[] args) {
        ExportSaveInternal(args[0]);
    }
}

