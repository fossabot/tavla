' Set encoding by setting locale to en-US
SetLocale 1033

dim Input
set Voice = CreateObject("SAPI.SpVoice")
Do While Not WScript.StdIn.atEndOfStream
    Input = WScript.Stdin.ReadLine()
    Voice.Speak Input, SVSFIsXML
Loop
WScript.Quit IfElse(Voice.WaitUntilDone(-1), 0, 1)

Function IfElse(bClause, sTrue, sFalse)
    If CBool(bClause) Then
        IfElse = sTrue
    Else 
        IfElse = sFalse
    End If
End Function
