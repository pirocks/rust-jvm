import javax.swing.*;
import java.util.ArrayList;
import java.util.Collection;

public class Swing {
    public static void main(String[] args) {
        JFrame test = new JFrame("test");
//        Collection<Integer> thing = new ArrayList<Integer>(){{
//            add(1);
//            add(2);
//        }};
//        for (Integer integer : thing) {
//            System.out.println(integer);
//        }
        test.setSize(100,100);
        test.setVisible(true);
    }
}
