public class Main {

	public Main() {
	}

	public synchronized void test(){
//		notifyAll();
		System.out.println("test called from" + Thread.currentThread().getName());
		try {
			Thread.sleep(1000);
		} catch (InterruptedException e) {
			e.printStackTrace();
		}
		System.out.println("test finished");
	}

	public static void main(String[] args) {
		final Main main = new Main();
		final Thread thread1 = new Thread(main::test);
		final Thread thread2 = new Thread(main::test);
		final Thread thread3 = new Thread(main::test);
		final Thread thread4 = new Thread(main::test);
		thread1.start();
		thread2.start();
		thread3.start();
		thread4.start();
	}
}
