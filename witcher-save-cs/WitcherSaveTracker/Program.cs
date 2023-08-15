using System;
using System.IO;
using System.Runtime.InteropServices;

using W3SavegameEditor.Core.Savegame;
using W3SavegameEditor.Core.Savegame.Variables;
using W3SavegameEditor.Core.Savegame.Values;
// using SaveFile;

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
        ExportSaveInternal(path);
    }

    static void ExportSaveInternal(string path) {
        var savegame = SavegameFile.Read(path);
        ExtractQuests(savegame);
    }

    static List<string> ExtractMapPinTags(SavegameFile savegame) {
        var ret = new List<string>();

        var common_map_manager = Array.Find(
            savegame.Variables, 
            x => x.Name == "CCommonMapManager"
        ) as BsVariable;

        foreach (var v in common_map_manager.Variables) {
          if (v.Name == "MapPinTag") {
              VlVariable vl = v as VlVariable;
              ret.Append(vl.Value.ToString());
          }
        }

        return ret;
    }

    static void ExtractQuests(SavegameFile savegame) {
        var journal_manager = Array.Find(
            savegame.Variables, 
            x => x.Name == "CJournalManager"
        ) as BsVariable;

        // JActiveEntries
        var active_entries = journal_manager.Variables[0] as BsVariable;
        Console.WriteLine(active_entries);
        Console.WriteLine(active_entries.Variables.Length);
    }

    public static void Main(string[] args) {
        ExportSaveInternal("../data/QuickSave.sav");
    }
}

        /* var savefile = new Witcher3SaveFile(path, Witcher3ReadLevel.Quick); */
        /* var quests = savefile.CJournalManager.Statuses */
        /*   .ConvertAll(status => new Quest {  */
        /*       Guid = status.PrimaryGUID,  */
        /*       Status = status.Status.ToString()  */
        /*   }); */
        /**/
        /* string data = ""; */
        /* foreach (var quest in quests) { */
        /*   data += quest.Guid + ";;" + quest.Status + "\n"; */
        /* } */
        /* File.WriteAllText("tw3savefile.csv", data); */
