; ModuleID = 'rt.c'
target datalayout = "e-m:o-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-apple-macosx10.10.0"

%struct.record = type { %struct.record_def* }
%struct.record_def = type { i32 }

@.str = private unnamed_addr constant [3 x i8] c"%p\00", align 1

; Function Attrs: nounwind ssp uwtable
define i8* @get_property() #0 {
  %1 = alloca i8*, align 8
  %2 = load i8** %1
  ret i8* %2
}

; Function Attrs: nounwind ssp uwtable
define i32 @main() #0 {
  %rec = alloca %struct.record*, align 8
  %randomptr = alloca i8*, align 8
  %rec2 = alloca %struct.record*, align 8
  call void @GC_init()
  %1 = call noalias i8* @GC_malloc(i64 48)
  %2 = bitcast i8* %1 to %struct.record*
  store %struct.record* %2, %struct.record** %rec, align 8
  %3 = load %struct.record** %rec, align 8
  %4 = call i32 (i8*, ...)* @printf(i8* getelementptr inbounds ([3 x i8]* @.str, i32 0, i32 0), %struct.record* %3)
  %5 = call noalias i8* @GC_malloc(i64 1)
  store i8* %5, i8** %randomptr, align 8
  %6 = load i8** %randomptr, align 8
  %7 = call i32 (i8*, ...)* @printf(i8* getelementptr inbounds ([3 x i8]* @.str, i32 0, i32 0), i8* %6)
  %8 = call noalias i8* @GC_malloc(i64 48)
  %9 = bitcast i8* %8 to %struct.record*
  store %struct.record* %9, %struct.record** %rec2, align 8
  %10 = load %struct.record** %rec2, align 8
  %11 = call i32 (i8*, ...)* @printf(i8* getelementptr inbounds ([3 x i8]* @.str, i32 0, i32 0), %struct.record* %10)
  ret i32 0
}

declare void @GC_init() #1

declare noalias i8* @GC_malloc(i64) #1

declare i32 @printf(i8*, ...) #1

attributes #0 = { nounwind ssp uwtable "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "stack-protector-buffer-size"="8" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #1 = { "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "stack-protector-buffer-size"="8" "unsafe-fp-math"="false" "use-soft-float"="false" }

!llvm.ident = !{!0}

!0 = metadata !{metadata !"Apple LLVM version 6.0 (clang-600.0.56) (based on LLVM 3.5svn)"}
