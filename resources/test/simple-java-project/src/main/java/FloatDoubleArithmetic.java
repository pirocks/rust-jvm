public class FloatDoubleArithmetic {

	long a;
	long b;
	long c;
	long d;
	double d_a;
	double d_b;
	double d_c;
	double d_d;
	float f_a;
	float f_b;
	float f_c;
	float f_d;


	public FloatDoubleArithmetic() {
		a = 0L;
		d_a = 5*6+4.0;
		b = 100L + 798;
		d_b = 5*6+4.0;
		f_a = (float) 7.0d;
		if(b < 1000){
			d_c = (double) 8.0f;
			d_d = d_b * 1000;
		}else {
			f_b = f_c;
			f_d = (float) d_b;
		}
		d_d = f_d;
	}
}
