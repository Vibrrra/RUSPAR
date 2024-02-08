function [hiir,iir_l, iir_r] = generate_iir(h, order_a, order_b)
% i = 52652;
% di = delta(i);
% h = [zeros(abs(di),2); squeeze(HRIR_MP(i,:,:)); zeros(abs(di),2)];
iir_l = dsp.IIRFilter;
iir_r = dsp.IIRFilter;

 [bcl,acl] = stmcb(h(:,1),order_b,order_a);
[bcr,acr] = stmcb(h(:,2),order_b,order_a);
iir_l.Numerator = bcl;
iir_l.Denominator = acl;
iir_r.Numerator = bcr;
iir_r.Denominator = acr;
hiir(:,1) = iir_l.impz(256);
hiir(:,2) = iir_r.impz(256);

end