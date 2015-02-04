; ModuleID = 'rt.c'
target datalayout = "e-m:o-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-apple-macosx10.10.0"

%struct.record = type { %struct.record_def* }
%struct.record_def = type { i32, i32 }
%struct.field_entry = type { i64, i64 }
%struct.value = type { i8, i64 }

@__func__.getProperty = private unnamed_addr constant [12 x i8] c"getProperty\00", align 1
@.str = private unnamed_addr constant [5 x i8] c"rt.c\00", align 1
@.str1 = private unnamed_addr constant [17 x i8] c"valueIsRecord(v)\00", align 1
@.str2 = private unnamed_addr constant [16 x i8] c"idx != s % size\00", align 1
@__func__.getMethod = private unnamed_addr constant [10 x i8] c"getMethod\00", align 1

; Function Attrs: nounwind readnone ssp uwtable
define i32 @valueIsDouble(i8 %v.coerce0, i64 %v.coerce1) #0 {
  %1 = icmp eq i8 %v.coerce0, 1
  %2 = zext i1 %1 to i32
  ret i32 %2
}

; Function Attrs: nounwind readnone ssp uwtable
define double @valueAsDouble(i8 %v.coerce0, i64 %v.coerce1) #0 {
  %1 = uitofp i64 %v.coerce1 to double
  ret double %1
}

; Function Attrs: nounwind readnone ssp uwtable
define i32 @valueIsRecord(i8 %v.coerce0, i64 %v.coerce1) #0 {
  %1 = icmp eq i8 %v.coerce0, 0
  %2 = zext i1 %1 to i32
  ret i32 %2
}

; Function Attrs: nounwind readnone ssp uwtable
define %struct.record* @valueAsRecord(i8 %v.coerce0, i64 %v.coerce1) #0 {
  %1 = inttoptr i64 %v.coerce1 to %struct.record*
  ret %struct.record* %1
}

; Function Attrs: nounwind readnone ssp uwtable
define i32 @valueIsBool(i8 %v.coerce0, i64 %v.coerce1) #0 {
  %1 = icmp eq i8 %v.coerce0, 3
  %2 = zext i1 %1 to i32
  ret i32 %2
}

; Function Attrs: nounwind readnone ssp uwtable
define i32 @valueAsBool(i8 %v.coerce0, i64 %v.coerce1) #0 {
  %1 = trunc i64 %v.coerce1 to i32
  ret i32 %1
}

; Function Attrs: nounwind ssp uwtable
define { i8, i64 } @getProperty(i8 %v.coerce0, i64 %v.coerce1, i64 %s) #1 {
  %1 = icmp eq i8 %v.coerce0, 0
  br i1 %1, label %3, label %2, !prof !1

; <label>:2                                       ; preds = %0
  tail call void @__assert_rtn(i8* getelementptr inbounds ([12 x i8]* @__func__.getProperty, i64 0, i64 0), i8* getelementptr inbounds ([5 x i8]* @.str, i64 0, i64 0), i32 70, i8* getelementptr inbounds ([17 x i8]* @.str1, i64 0, i64 0)) #4
  unreachable

; <label>:3                                       ; preds = %0
  %4 = inttoptr i64 %v.coerce1 to %struct.record*
  %5 = getelementptr inbounds %struct.record* %4, i64 0, i32 0
  %6 = load %struct.record_def** %5, align 8, !tbaa !2
  %7 = getelementptr inbounds %struct.record_def* %6, i64 0, i32 0
  %8 = load i32* %7, align 4, !tbaa !7
  %9 = zext i32 %8 to i64
  %10 = urem i64 %s, %9
  %11 = trunc i64 %10 to i32
  %12 = getelementptr inbounds %struct.record_def* %6, i64 1
  %13 = bitcast %struct.record_def* %12 to %struct.field_entry*
  br label %14

; <label>:14                                      ; preds = %19, %3
  %idx.0 = phi i32 [ %11, %3 ], [ %21, %19 ]
  %15 = zext i32 %idx.0 to i64
  %16 = getelementptr inbounds %struct.field_entry* %13, i64 %15, i32 0
  %17 = load i64* %16, align 8, !tbaa !10
  %18 = icmp eq i64 %17, %s
  br i1 %18, label %25, label %19

; <label>:19                                      ; preds = %14
  %20 = add i32 %idx.0, 1
  %21 = urem i32 %20, %8
  %22 = zext i32 %21 to i64
  %23 = icmp eq i64 %22, %10
  br i1 %23, label %24, label %14, !prof !14

; <label>:24                                      ; preds = %19
  tail call void @__assert_rtn(i8* getelementptr inbounds ([12 x i8]* @__func__.getProperty, i64 0, i64 0), i8* getelementptr inbounds ([5 x i8]* @.str, i64 0, i64 0), i32 84, i8* getelementptr inbounds ([16 x i8]* @.str2, i64 0, i64 0)) #4
  unreachable

; <label>:25                                      ; preds = %14
  %26 = getelementptr inbounds %struct.field_entry* %13, i64 %15, i32 1
  %27 = load i64* %26, align 8, !tbaa !15
  %28 = getelementptr inbounds %struct.record* %4, i64 1
  %29 = bitcast %struct.record* %28 to %struct.value*
  %30 = getelementptr inbounds %struct.value* %29, i64 %27, i32 0
  %31 = load i8* %30, align 8
  %32 = getelementptr inbounds %struct.value* %29, i64 %27, i32 1
  %33 = load i64* %32, align 8
  %34 = insertvalue { i8, i64 } undef, i8 %31, 0
  %35 = insertvalue { i8, i64 } %34, i64 %33, 1
  ret { i8, i64 } %35
}

; Function Attrs: noreturn
declare void @__assert_rtn(i8*, i8*, i32, i8*) #2

; Function Attrs: nounwind ssp uwtable
define i8* @getMethod(i8 %v.coerce0, i64 %v.coerce1, i64 %s) #1 {
  %1 = icmp eq i8 %v.coerce0, 0
  br i1 %1, label %3, label %2, !prof !1

; <label>:2                                       ; preds = %0
  tail call void @__assert_rtn(i8* getelementptr inbounds ([10 x i8]* @__func__.getMethod, i64 0, i64 0), i8* getelementptr inbounds ([5 x i8]* @.str, i64 0, i64 0), i32 91, i8* getelementptr inbounds ([17 x i8]* @.str1, i64 0, i64 0)) #4
  unreachable

; <label>:3                                       ; preds = %0
  %4 = inttoptr i64 %v.coerce1 to %struct.record*
  %5 = getelementptr inbounds %struct.record* %4, i64 0, i32 0
  %6 = load %struct.record_def** %5, align 8, !tbaa !2
  %7 = getelementptr inbounds %struct.record_def* %6, i64 0, i32 1
  %8 = load i32* %7, align 4, !tbaa !16
  %9 = zext i32 %8 to i64
  %10 = urem i64 %s, %9
  %11 = trunc i64 %10 to i32
  %12 = getelementptr inbounds %struct.record_def* %6, i64 1
  %13 = bitcast %struct.record_def* %12 to %struct.field_entry*
  %14 = getelementptr inbounds %struct.record_def* %6, i64 0, i32 0
  %15 = load i32* %14, align 4, !tbaa !7
  %16 = zext i32 %15 to i64
  br label %17

; <label>:17                                      ; preds = %22, %3
  %idx.0 = phi i32 [ %11, %3 ], [ %24, %22 ]
  %18 = zext i32 %idx.0 to i64
  %.sum = add i64 %18, %16
  %19 = getelementptr inbounds %struct.field_entry* %13, i64 %.sum, i32 0
  %20 = load i64* %19, align 8, !tbaa !17
  %21 = icmp eq i64 %20, %s
  br i1 %21, label %28, label %22

; <label>:22                                      ; preds = %17
  %23 = add i32 %idx.0, 1
  %24 = urem i32 %23, %8
  %25 = zext i32 %24 to i64
  %26 = icmp eq i64 %25, %10
  br i1 %26, label %27, label %17, !prof !14

; <label>:27                                      ; preds = %22
  tail call void @__assert_rtn(i8* getelementptr inbounds ([10 x i8]* @__func__.getMethod, i64 0, i64 0), i8* getelementptr inbounds ([5 x i8]* @.str, i64 0, i64 0), i32 106, i8* getelementptr inbounds ([16 x i8]* @.str2, i64 0, i64 0)) #4
  unreachable

; <label>:28                                      ; preds = %17
  %29 = getelementptr inbounds %struct.field_entry* %13, i64 %.sum, i32 1
  %30 = bitcast i64* %29 to i8**
  %31 = load i8** %30, align 8, !tbaa !19
  ret i8* %31
}

; Function Attrs: nounwind ssp uwtable
define { i8, i64 } @allocRecord(i64 %size) #1 {
  %1 = tail call noalias i8* @GC_malloc(i64 %size) #5
  %2 = ptrtoint i8* %1 to i64
  %3 = insertvalue { i8, i64 } { i8 0, i64 undef }, i64 %2, 1
  ret { i8, i64 } %3
}

declare noalias i8* @GC_malloc(i64) #3

; Function Attrs: nounwind ssp uwtable
define i32 @main() #1 {
  tail call void @GC_init() #5
  tail call void (...)* @__ducky__main() #5
  ret i32 0
}

declare void @GC_init() #3

declare void @__ducky__main(...) #3

attributes #0 = { nounwind readnone ssp uwtable "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "stack-protector-buffer-size"="8" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #1 = { nounwind ssp uwtable "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "stack-protector-buffer-size"="8" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #2 = { noreturn "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "stack-protector-buffer-size"="8" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #3 = { "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "stack-protector-buffer-size"="8" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #4 = { noreturn nounwind }
attributes #5 = { nounwind }

!llvm.ident = !{!0}

!0 = metadata !{metadata !"Apple LLVM version 6.0 (clang-600.0.56) (based on LLVM 3.5svn)"}
!1 = metadata !{metadata !"branch_weights", i32 64, i32 4}
!2 = metadata !{metadata !3, metadata !4, i64 0}
!3 = metadata !{metadata !"record", metadata !4, i64 0}
!4 = metadata !{metadata !"any pointer", metadata !5, i64 0}
!5 = metadata !{metadata !"omnipotent char", metadata !6, i64 0}
!6 = metadata !{metadata !"Simple C/C++ TBAA"}
!7 = metadata !{metadata !8, metadata !9, i64 0}
!8 = metadata !{metadata !"record_def", metadata !9, i64 0, metadata !9, i64 4}
!9 = metadata !{metadata !"int", metadata !5, i64 0}
!10 = metadata !{metadata !11, metadata !12, i64 0}
!11 = metadata !{metadata !"field_entry", metadata !12, i64 0, metadata !13, i64 8}
!12 = metadata !{metadata !"long long", metadata !5, i64 0}
!13 = metadata !{metadata !"long", metadata !5, i64 0}
!14 = metadata !{metadata !"branch_weights", i32 4, i32 64}
!15 = metadata !{metadata !11, metadata !13, i64 8}
!16 = metadata !{metadata !8, metadata !9, i64 4}
!17 = metadata !{metadata !18, metadata !12, i64 0}
!18 = metadata !{metadata !"mthd_entry", metadata !12, i64 0, metadata !4, i64 8}
!19 = metadata !{metadata !18, metadata !4, i64 8}
