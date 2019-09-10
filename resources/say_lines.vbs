' Set encoding by setting locale to en-US
SetLocale 1033

Const SAFT48kHz16BitStereo = 39
Const SSFMCreateForWrite = 3 ' Creates file even if file exists and so destroys or overwrites the existing file

Dim Input
Dim FileStream, Filename

ToFile = false
Set Voice = CreateObject("SAPI.SpVoice")
Do While Not WScript.StdIn.atEndOfStream
    Input = WScript.Stdin.ReadLine()

    If Input = "53377b5f-60a2-4c05-a4eb-55de35452a2b" Then
        ' when magic UUID received, we are supposed to write to the file on the next line
        File = WScript.Stdin.ReadLine()
        Set FileStream = CreateObject("SAPI.SpFileStream")
        FileStream.Format.Type = SAFT48kHz16BitStereo
        FileStream.Open File, SSFMCreateForWrite
        Set Voice.AudioOutputStream = FileStream
        ToFile = true
    Else
        ' otherwise just speak the text
        Voice.Speak Input, SVSFIsXML
    End If
Loop

If Voice.WaitUntilDone(-1) Then
    If ToFile Then
        FileStream.Close
    End If
    WScript.Quit 0
Else
    WScript.Quit 1
End If
