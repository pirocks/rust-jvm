import java.io.*;
import java.nio.file.Files;
import java.nio.file.Path;

public class IOExamples {
    public static void main(String[] args) throws IOException {
        Path tempDir = Files.createTempDirectory("rust-jvm-demo");
        final File new_file = tempDir.resolve("new_file").toFile();
        if(!new_file.createNewFile()){
            throw new IOException();
        }
        final DataOutputStream outputStream = new DataOutputStream(new FileOutputStream(new_file));
        final String testString = "This is a test String";
        outputStream.writeUTF(testString);
        outputStream.close();
        final DataInputStream inputStream = new DataInputStream(new FileInputStream(new_file));
        final String s = inputStream.readUTF();
        if(s.equals(testString)){
            System.out.println("success");
            System.exit(0);
        }else {
            System.out.println("not successful");
            System.exit(-1);
        }
    }
}
