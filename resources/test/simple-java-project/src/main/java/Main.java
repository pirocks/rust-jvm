
public class Main {

	public long a_thing = 0;

	public Main() {
		long a_long_var = 1L;
		long a_long_var_2 = 1L;
		long other = a_long_var * a_long_var_2;
	}

	public static void main(String[] args) throws NoSuchFieldException {
		Main.class.getDeclaredField("a_thing");
		System.out.println(int.class.getName());
	    System.out.println("here");
	    System.out.println("also here");
	    int a_var = 0;
	    if(Math.random() > 0){
			int another_var = a_var + 1;
			char a_char_var = 'g';
			double a_double  = a_var;
		}else {
	    	a_var += 1;
		}
    }
}
