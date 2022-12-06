//import com.sun.jdi.*;
//import com.sun.jdi.connect.IllegalConnectorArgumentsException;
//import com.sun.jdi.event.*;
//import com.sun.jdi.request.BreakpointRequest;
//import com.sun.tools.jdi.GenericAttachingConnector;
//import com.sun.tools.jdi.SocketAttachingConnector;
//
//import java.io.IOException;
//import java.util.List;
//
//public class DebuggingClass {
//    public static void main(String[] args) throws IOException, IllegalConnectorArgumentsException, AbsentInformationException, IncompatibleThreadStateException {
//        final GenericAttachingConnector connector = new SocketAttachingConnector();
//        final VirtualMachine attached = connector.attach("localhost:5005", connector.defaultArguments());
//        for (ReferenceType aClass : attached.allClasses()) {
//           /* if (!aClass.name().equals("int") &&
//                    !aClass.name().equals("boolean") &&
//                    !aClass.name().equals("byte") &&
//                    !aClass.name().equals("short") &&
//                    !aClass.name().equals("float") &&
//                    !aClass.name().equals("double") &&
//                    !aClass.name().equals("long") &&
//                    !aClass.name().equals("char")) {
//                for (Method method : aClass.allMethods()) {
//                    System.out.println(method.name());
//                }
//            }*/
//            System.out.println(aClass.name());
//        }
////        attached.setDebugTraceMode(VirtualMachine.TRACE_ALL);
////        attached.suspend();
//        final ReferenceType swing = attached.classesByName("Swing").get(0);
//        final Method method = swing.methods().stream().filter(method1 -> method1.name().equals("main")).findFirst().get();
//        final Location location = method.allLineLocations().get(1);
//        final BreakpointRequest breakpointRequest = attached.eventRequestManager().createBreakpointRequest(location);
//        breakpointRequest.enable();
//
//        final List<ThreadReference> threadReferences = attached.allThreads();
//        for (ThreadReference thread : threadReferences) {
//            System.out.println(thread.name());
//            for (StackFrame frame : thread.frames()) {System.out.println(frame.location());
//            }
//            thread.resume();
//        }
//
//        attached.resume();
////        EventQueue queue = attached.eventQueue();
////        while (true) {
////            EventSet eventSet = null;
////            try {
////                eventSet = queue.remove();
////            } catch (InterruptedException e) {
////                e.printStackTrace();
////            }
////            EventIterator it = eventSet.eventIterator();
////            while (it.hasNext()) {
////                Event event = it.nextEvent();
////                if (event instanceof BreakpointEvent) {
////                    final BreakpointEvent event1 = (BreakpointEvent) event;
////                    System.out.println(event1.thread().name());
////                    final List<StackFrame> frames = event1.thread().frames();
////                    frames.forEach(stackFrame -> {
////                        final List<LocalVariable> localVariableList;
////                        try {
////                            localVariableList = stackFrame.visibleVariables();
////                            localVariableList.forEach(localVariable -> {
////                                System.out.println(localVariable.name());
////                                try {
////                                    System.out.println(localVariable.type());
////                                } catch (ClassNotLoadedException e) {
////                                    e.printStackTrace();
////                                }
////                                System.out.println(((ArrayReference) stackFrame.getValue(localVariable)).getValue(1));
////                                try {
////                                    System.out.println(stackFrame.visibleVariableByName("args"));
////                                } catch (AbsentInformationException e) {
////                                    e.printStackTrace();
////                                }
////                            });
////                        } catch (AbsentInformationException e) {
////                            e.printStackTrace();
////                        }
////
////                    });
////
////
////                    final List<ThreadReference> allThreads = attached.allThreads();
////                    allThreads.forEach(threadReference -> {
////                        final String name = threadReference.name();
////                        System.out.println(name);
////
////
////                        try {
////                            threadReference.frames().forEach(stackFrame -> {
////                                try {
////                                    stackFrame.visibleVariables().forEach(localVariable -> {
////                                        System.out.println(localVariable.name());
////                                    });
////                                } catch (AbsentInformationException e) {
////                                    e.printStackTrace();
////                                }
////                                stackFrame.getArgumentValues().forEach(value -> {
////                                    System.out.println(value.type());
////                                });
////                            });
////                        } catch (IncompatibleThreadStateException e) {
////                            e.printStackTrace();
////                        }
////                    });
////                }
////            }
////        }
//////        for (ThreadReference thread : threads) {
//////            if(thread.name().equals("Main")) {
////////                thread.interrupt();
//////                thread.interrupt();
//////                thread.suspend();
//////                System.out.println(thread.name());
//////                System.out.println(thread.status());
//////                System.out.println("Suspended:" + thread.isSuspended());
//////                try {
//////                    System.out.println(thread.isSuspended());
//////                    for (StackFrame frame : thread.frames()) {
//////                        try {
//////                            for (LocalVariable variable : frame.visibleVariables()) {
//////                                System.out.println(variable.name());
//////                                System.out.println(frame.getValue(variable));
//////                                System.out.println(variable.isArgument());
//////                                System.out.println(variable.genericSignature());
//////
//////                            }
//////                        } catch (AbsentInformationException e) {
//////                            e.printStackTrace();
//////                        }
//////                        System.out.println(frame.thisObject());
//////                        System.out.println(frame.thread());
//////                        try {
//////                            System.out.println(frame.location().lineNumber());
//////                        } catch (InternalError e) {
//////                            e.printStackTrace();
//////                        }
//////                    }
//////                    System.out.println(thread.frameCount());
//////                } catch (IncompatibleThreadStateException e) {
//////                    e.printStackTrace();
//////                }
//////
//////                System.out.println(thread.name());
//////            }
//////        }
////
////
//    }
//}

import java.io.*;
import java.lang.reflect.*;
import java.security.*;

public class DebuggingClass {

    public static void main(String[] args) throws Exception {
        int access = DebuggingClass.class.getModifiers();
        System.out.println(access);
        if (java.lang.reflect.Modifier.isSynchronized(access))
            throw new Exception("ACC_SUPER bit is not being stripped");

        //        final File file = new File(DebuggingClass.class.getResource("ToLowerCase.class").getPath());
//        byte[] bytes = new byte[(int) file.length()];
//        final DataInputStream inputStream = new DataInputStream(new FileInputStream(file));
//        inputStream.readFully(bytes);
//        System.out.println(Arrays.toString(bytes));

//        for (Method declaredMethod : void.class.getDeclaredMethods()) {
//            System.out.println(declaredMethod);
//        }
//        System.identityHashCode(null);
//        test(new BackingStoreException("Hi"));
//        TimeZone tz = TimeZone.getTimeZone("Asia/Taipei");
//        Locale tzLocale = new Locale("ja");
//        System.out.println(new Test().getCandidateLocales("sun.util.resources.TimeZoneNames",tzLocale).size());
//        System.out.println(ResourceBundle.Control.FORMAT_DEFAULT);
//        System.out.println(Arrays.toString(TimeZoneNameUtility.retrieveDisplayNames(tz.getID(), tzLocale)));
//        System.out.println(tz.getDisplayName(false, TimeZone.LONG, tzLocale));
//        System.out.println(Arrays.toString((Object[]) new TimeZoneNames_ja().getObject("Asia/Taipei")));
//        System.getProperties().forEach((key, val) -> {
//            System.out.println("key:" + key);
//            System.out.println("val:" + val);
//        });
    }

//    private static void extracted() {
//        final StringBuilder stringBuilder = new StringBuilder();
//        final char[] zeros = ZEROES.toCharArray();
//        for (char c : zeros) {
//            stringBuilder.append((char) ((int) c + 9));
//        }
//        System.out.println(zeros.length);
//        final char[] nines = stringBuilder.toString().toCharArray();
//        System.out.println(nines.length);
//        System.out.println(stringBuilder);
//        matcher(zeros, nines);
//    }
//
//    private static void matcher(char[] zeros, char[] nines) {
//        for (int i = 0; i < zeros.length; ++i) {
//            checkArgument(zeros[i] <= nines[i]);
//            if (i + 1 < zeros.length) {
//                System.out.println((int)nines[i]);
//                System.out.println((int)zeros[i + 1]);
//                checkArgument(nines[i] < zeros[i + 1]);
//            }
//        }
//    }
//
//    private static void checkArgument(boolean b) {
//        if (!b) {
//            throw new IllegalArgumentException();
//        }
//    }

}
